//! HashMap avec expiration automatique des entrées basée sur le temps
//! Utile pour le throttling et le caching avec TTL

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::time::{Duration, Instant};

/// Entrée dans le TimedHashMap avec timestamp d'insertion
#[derive(Debug, Clone)]
struct TimedEntry<V> {
    value: V,
    inserted_at: Instant,
}

/// HashMap qui supprime automatiquement les entrées après un délai (TTL)
#[derive(Debug)]
pub struct TimedHashMap<K, V>
where
    K: Eq + Hash,
{
    map: HashMap<K, TimedEntry<V>>,
    ttl: Duration,
    insert_count: usize,
    cleanup_interval: usize,
}

impl<K, V> TimedHashMap<K, V>
where
    K: Eq + Hash + Clone,
{
    /// Crée un nouveau TimedHashMap avec le TTL spécifié
    pub fn new(ttl: Duration) -> Self {
        Self::with_cleanup_interval(ttl, 100)
    }

    /// Crée un nouveau TimedHashMap avec le TTL spécifié et l'intervalle de nettoyage
    ///
    /// # Arguments
    /// * `ttl` - Durée de vie des entrées
    /// * `cleanup_interval` - Nombre d'insertions entre chaque nettoyage automatique
    pub fn with_cleanup_interval(ttl: Duration, cleanup_interval: usize) -> Self {
        Self {
            map: HashMap::new(),
            ttl,
            insert_count: 0,
            cleanup_interval,
        }
    }

    /// Crée un nouveau TimedHashMap avec capacité initiale et TTL
    pub fn with_capacity(capacity: usize, ttl: Duration) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            ttl,
            insert_count: 0,
            cleanup_interval: 100,
        }
    }

    /// Insère une valeur avec timestamp actuel
    /// Nettoie automatiquement les entrées expirées tous les N insertions
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let entry = TimedEntry {
            value,
            inserted_at: Instant::now(),
        };

        self.insert_count += 1;

        // Nettoyage automatique tous les N insertions
        if self.insert_count % self.cleanup_interval == 0 {
            self.cleanup_expired();
        }

        self.map.insert(key, entry).map(|e| e.value)
    }

    /// Récupère une valeur si elle existe et n'est pas expirée
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.get(key).and_then(|entry| {
            if entry.inserted_at.elapsed() < self.ttl {
                Some(&entry.value)
            } else {
                None
            }
        })
    }

    /// Récupère une valeur mutable si elle existe et n'est pas expirée
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let ttl = self.ttl;
        self.map.get_mut(key).and_then(|entry| {
            if entry.inserted_at.elapsed() < ttl {
                Some(&mut entry.value)
            } else {
                None
            }
        })
    }

    /// Vérifie si une clé existe et n'est pas expirée
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Supprime une entrée
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.remove(key).map(|e| e.value)
    }

    /// Nettoie toutes les entrées expirées
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.map
            .retain(|_, entry| now.duration_since(entry.inserted_at) < self.ttl);
    }

    /// Retourne le nombre total d'entrées (incluant celles expirées)
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Retourne le nombre d'entrées non expirées
    pub fn active_len(&self) -> usize {
        let now = Instant::now();
        self.map
            .values()
            .filter(|entry| now.duration_since(entry.inserted_at) < self.ttl)
            .count()
    }

    /// Vérifie si la map est vide (ignore les entrées expirées)
    pub fn is_empty(&self) -> bool {
        self.active_len() == 0
    }

    /// Vide complètement la map
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl<K, V> Default for TimedHashMap<K, V>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new(Duration::from_secs(3600)) // 1 heure par défaut
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_insert_and_get() {
        let mut map = TimedHashMap::new(Duration::from_secs(10));
        map.insert("key1", 42);

        assert_eq!(map.get(&"key1"), Some(&42));
        assert_eq!(map.get(&"key2"), None);
    }

    #[test]
    fn test_expiration() {
        let mut map = TimedHashMap::new(Duration::from_millis(50));
        map.insert("key1", 42);

        assert_eq!(map.get(&"key1"), Some(&42));

        thread::sleep(Duration::from_millis(100));

        assert_eq!(map.get(&"key1"), None);
    }

    #[test]
    fn test_cleanup_expired() {
        let mut map = TimedHashMap::new(Duration::from_millis(50));
        map.insert("key1", 1);
        map.insert("key2", 2);
        map.insert("key3", 3);

        assert_eq!(map.len(), 3);

        thread::sleep(Duration::from_millis(100));

        // Les entrées sont encore dans la map physiquement
        assert_eq!(map.len(), 3);

        // Mais active_len ne compte que les non-expirées
        assert_eq!(map.active_len(), 0);

        // Cleanup supprime les expirées
        map.cleanup_expired();
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_partial_expiration() {
        let mut map = TimedHashMap::new(Duration::from_millis(100));
        map.insert("key1", 1);

        thread::sleep(Duration::from_millis(50));
        map.insert("key2", 2);

        thread::sleep(Duration::from_millis(60));

        // key1 expirée (110ms), key2 valide (60ms)
        assert_eq!(map.get(&"key1"), None);
        assert_eq!(map.get(&"key2"), Some(&2));

        map.cleanup_expired();
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut map = TimedHashMap::new(Duration::from_secs(10));
        map.insert("key1", 42);

        assert_eq!(map.remove(&"key1"), Some(42));
        assert_eq!(map.get(&"key1"), None);
        assert_eq!(map.remove(&"key1"), None);
    }

    #[test]
    fn test_get_mut() {
        let mut map = TimedHashMap::new(Duration::from_secs(10));
        map.insert("key1", 42);

        if let Some(value) = map.get_mut(&"key1") {
            *value = 100;
        }

        assert_eq!(map.get(&"key1"), Some(&100));
    }
}
