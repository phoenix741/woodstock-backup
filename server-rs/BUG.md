# BUG Refacto rust

[X] Ne s'affiche pas tout de suite au démarrage.
[X] Pas d'ajout de la tâche par la websocket (F5 obligatoire pour avoir la tache et le rafraichissement).
[?] 100% CPU sur api_server (en mode debug) (augmentation avec le nombre de message) - 12% CPU sur job_worker
[/] Dead lock entre fsck et backup (à confirmer) - à priori ralenti par la partie progression ...
[?] Long apalis job qui continue mais qui est relancer du point de vue d'apalis => job en double (c'est le bordel) => mais pas de lock => corruption de tout --> fournir un lock au niveau du host, backup number ?
[X] J'ai plein de job:progress:XXXX pour le job de cleanup. Pourquoi ?
[X] Fuite mémoire (après plusieurs heures de fonctionnement j'arrive à 12Go de RAM sur api_server)
[X] progress.rs fuite mémoire sur le last_published
[ ] Pas de cache
