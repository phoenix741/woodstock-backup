use eyre::Result;
use tracing::error;
use windows::Win32::NetworkManagement::WindowsFirewall::INetFwRule;
use windows::Win32::NetworkManagement::WindowsFirewall::{
    INetFwPolicy2, NetFwPolicy2, NET_FW_ACTION, NET_FW_ACTION_ALLOW, NET_FW_ACTION_BLOCK,
    NET_FW_ACTION_MAX, NET_FW_IP_PROTOCOL_ANY, NET_FW_IP_PROTOCOL_TCP, NET_FW_IP_PROTOCOL_UDP,
    NET_FW_PROFILE2_PUBLIC, NET_FW_RULE_DIRECTION, NET_FW_RULE_DIR_IN, NET_FW_RULE_DIR_OUT,
};
use windows::Win32::NetworkManagement::WindowsFirewall::{NetFwRule, NET_FW_IP_PROTOCOL};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};

/// Network protocol used for the firewall rule.
#[derive(Clone, Default)]
pub enum Protocols {
    /// TCP protocol.
    Tcp,
    /// UDP protocol.
    Udp,
    /// Any protocol (default value).
    #[default]
    Any,
}

impl From<&NET_FW_IP_PROTOCOL> for Protocols {
    fn from(w: &NET_FW_IP_PROTOCOL) -> Protocols {
        if w.eq(&NET_FW_IP_PROTOCOL_TCP) {
            Protocols::Tcp
        } else if w.eq(&NET_FW_IP_PROTOCOL_UDP) {
            Protocols::Udp
        } else {
            Protocols::Any
        }
    }
}

impl From<i32> for Protocols {
    fn from(w: i32) -> Protocols {
        let protocol = NET_FW_IP_PROTOCOL(w);
        (&protocol).into()
    }
}

impl From<&Protocols> for NET_FW_IP_PROTOCOL {
    fn from(w: &Protocols) -> NET_FW_IP_PROTOCOL {
        match w {
            Protocols::Tcp => NET_FW_IP_PROTOCOL_TCP,
            Protocols::Udp => NET_FW_IP_PROTOCOL_UDP,
            Protocols::Any => NET_FW_IP_PROTOCOL_ANY,
        }
    }
}

/// Traffic direction for the firewall rule.
#[derive(Clone, Default)]
pub enum Directions {
    /// Incoming traffic.
    In,
    /// Outgoing traffic.
    Out,
    /// Any direction (default value).
    #[default]
    Any,
}

impl From<&NET_FW_RULE_DIRECTION> for Directions {
    fn from(w: &NET_FW_RULE_DIRECTION) -> Directions {
        if w.eq(&NET_FW_RULE_DIR_IN) {
            Directions::In
        } else if w.eq(&NET_FW_RULE_DIR_OUT) {
            Directions::Out
        } else {
            Directions::Any
        }
    }
}

impl From<i32> for Directions {
    fn from(w: i32) -> Directions {
        let direction = NET_FW_RULE_DIRECTION(w);
        (&direction).into()
    }
}

impl From<&Directions> for NET_FW_RULE_DIRECTION {
    fn from(w: &Directions) -> NET_FW_RULE_DIRECTION {
        match w {
            Directions::In => NET_FW_RULE_DIR_IN,
            Directions::Out => NET_FW_RULE_DIR_OUT,
            Directions::Any => NET_FW_RULE_DIR_IN,
        }
    }
}

/// Action to apply to the firewall rule.
#[derive(Clone, Default)]
pub enum Actions {
    /// Block traffic (default value).
    #[default]
    Block,
    /// Allow traffic.
    Allow,
    /// Max action (Windows API specific).
    Max,
}

impl From<&NET_FW_ACTION> for Actions {
    fn from(w: &NET_FW_ACTION) -> Actions {
        if w.eq(&NET_FW_ACTION_BLOCK) {
            Actions::Block
        } else if w.eq(&NET_FW_ACTION_ALLOW) {
            Actions::Allow
        } else {
            Actions::Max
        }
    }
}

impl From<i32> for Actions {
    fn from(w: i32) -> Actions {
        let action = NET_FW_ACTION(w);
        (&action).into()
    }
}

impl From<&Actions> for NET_FW_ACTION {
    fn from(w: &Actions) -> NET_FW_ACTION {
        match w {
            Actions::Block => NET_FW_ACTION_BLOCK,
            Actions::Allow => NET_FW_ACTION_ALLOW,
            Actions::Max => NET_FW_ACTION_MAX,
        }
    }
}

/// Represents a Windows firewall rule.
#[derive(Default, Clone)]
pub struct FwRule {
    /// Rule name.
    pub name: String,
    /// Rule description.
    pub description: String,
    /// Associated application name.
    pub app_name: String,
    /// Associated service name.
    pub service_name: String,
    /// Protocol used.
    pub protocol: Protocols,
    /// ICMP type (if applicable).
    pub icmp_type: String,
    /// Local ports concerned.
    pub local_ports: String,
    /// Remote ports concerned.
    pub remote_ports: String,
    /// Local addresses concerned.
    pub local_adresses: String,
    /// Remote addresses concerned.
    pub remote_addresses: String,
    /// Profile 1 (Windows Firewall specific).
    pub profile1: String,
    /// Profile 2 (Windows Firewall specific).
    pub profile2: String,
    /// Profile 3 (Windows Firewall specific).
    pub profile3: String,
    /// Traffic direction.
    pub direction: Directions,
    /// Action to apply.
    pub action: Actions,
    /// Interface types concerned.
    pub interface_types: String,
    /// Interfaces concerned.
    pub interfaces: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Rule grouping.
    pub grouping: String,
    /// Whether edge traversal is enabled.
    pub edge_traversal: bool,
}

/// RAII helper to uninitialize COM on drop.
struct DropUninitializeCom;

impl Drop for DropUninitializeCom {
    fn drop(&mut self) {
        uninitialize_com();
    }
}

/// Initializes the COM model.
///
/// # Errors
/// Returns an error if COM initialization fails.
fn initialize_com() -> Result<()> {
    unsafe {
        CoInitializeEx(Some(std::ptr::null_mut()), COINIT_APARTMENTTHREADED).ok()?;
    }

    Ok(())
}

/// Uninitializes the COM model.
fn uninitialize_com() {
    unsafe {
        CoUninitialize();
    }
}

/// Creates a Windows firewall rule.
///
/// # Errors
/// Returns an error if rule creation fails.
pub fn create_firewall_rule(rule: &FwRule) -> Result<()> {
    let _ = DropUninitializeCom;
    initialize_com()?;

    unsafe {
        let fw_policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)?;
        let rules = fw_policy.Rules()?;
        let mut current_profiles_bit_mask = fw_policy.CurrentProfileTypes()?;

        if (current_profiles_bit_mask & NET_FW_PROFILE2_PUBLIC.0) != 0
            && (current_profiles_bit_mask != NET_FW_PROFILE2_PUBLIC.0)
        {
            current_profiles_bit_mask ^= NET_FW_PROFILE2_PUBLIC.0;
        }

        let protocole: NET_FW_IP_PROTOCOL = (&rule.protocol).into();
        let direction: NET_FW_RULE_DIRECTION = (&rule.direction).into();
        let action: NET_FW_ACTION = (&rule.action).into();

        let fw_rule: INetFwRule = CoCreateInstance(&NetFwRule, None, CLSCTX_INPROC_SERVER)?;
        if let Err(err) = fw_rule.SetName(&windows::core::BSTR::from(rule.name.as_str())) {
            error!("Error setting rule name: {}", err);
        }
        if let Err(err) =
            fw_rule.SetDescription(&windows::core::BSTR::from(rule.description.as_str()))
        {
            error!("Error setting rule description: {}", err);
        }
        if let Err(err) =
            fw_rule.SetApplicationName(&windows::core::BSTR::from(rule.app_name.as_str()))
        {
            error!("Error setting rule application name: {}", err);
        }
        if let Err(err) =
            fw_rule.SetServiceName(&windows::core::BSTR::from(rule.service_name.as_str()))
        {
            error!("Error setting rule service name: {}", err);
        }
        if let Err(err) = fw_rule.SetProtocol(protocole.0) {
            error!("Error setting rule protocol: {}", err);
        }
        if let Err(err) = fw_rule.SetIcmpTypesAndCodes(&windows::core::BSTR::from(&rule.icmp_type))
        {
            error!("Error setting rule icmp type: {}", err);
        }
        if let Err(err) = fw_rule.SetLocalPorts(&windows::core::BSTR::from(&rule.local_ports)) {
            error!("Error setting rule local ports: {}", err);
        }
        if let Err(err) = fw_rule.SetRemotePorts(&windows::core::BSTR::from(&rule.remote_ports)) {
            error!("Error setting rule remote ports: {}", err);
        }
        if let Err(err) =
            fw_rule.SetLocalAddresses(&windows::core::BSTR::from(&rule.local_adresses))
        {
            error!("Error setting rule local addresses: {}", err);
        }
        if let Err(err) =
            fw_rule.SetRemoteAddresses(&windows::core::BSTR::from(&rule.remote_addresses))
        {
            error!("Error setting rule remote addresses: {}", err);
        }
        if let Err(err) = fw_rule.SetDirection(direction) {
            error!("Error setting rule direction: {}", err);
        }
        if let Err(err) = fw_rule.SetAction(action) {
            error!("Error setting rule action: {}", err);
        }
        if let Err(err) =
            fw_rule.SetInterfaceTypes(&windows::core::BSTR::from(&rule.interface_types))
        {
            error!("Error setting rule interface types: {}", err);
        }
        if let Err(err) = fw_rule.SetEnabled(if rule.enabled {
            windows::Win32::Foundation::VARIANT_TRUE
        } else {
            windows::Win32::Foundation::VARIANT_FALSE
        }) {
            error!("Error setting rule enabled: {}", err);
        }
        if let Err(err) = fw_rule.SetGrouping(&windows::core::BSTR::from(&rule.grouping)) {
            error!("Error setting rule grouping: {}", err);
        }
        if let Err(err) = fw_rule.SetProfiles(current_profiles_bit_mask) {
            error!("Error setting rule profiles: {}", err);
        }
        if let Err(err) = fw_rule.SetEdgeTraversal(if rule.edge_traversal {
            windows::Win32::Foundation::VARIANT_TRUE
        } else {
            windows::Win32::Foundation::VARIANT_FALSE
        }) {
            error!("Error setting rule edge traversal: {}", err);
        }

        rules.Add(&fw_rule)?;
    }

    Ok(())
}

/// Deletes a Windows firewall rule by name.
///
/// # Errors
/// Returns an error if deletion fails.
pub fn delete_firewall_rule(name: &str) -> Result<()> {
    let _ = DropUninitializeCom;
    initialize_com()?;

    unsafe {
        let fw_policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)?;
        let rules = fw_policy.Rules()?;
        rules.Remove(&windows::core::BSTR::from(name))?;
    }

    Ok(())
}

/// Enables or disables a Windows firewall rule by name.
///
/// # Errors
/// Returns an error if the operation fails.
pub fn enable_fw_rule(name: &str, enabled: bool) -> Result<()> {
    let _ = DropUninitializeCom;
    initialize_com()?;

    unsafe {
        let fw_policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)?;
        let rules = fw_policy.Rules()?;
        let rule = rules.Item(&windows::core::BSTR::from(name))?;
        rule.SetEnabled(if enabled {
            windows::Win32::Foundation::VARIANT_TRUE
        } else {
            windows::Win32::Foundation::VARIANT_FALSE
        })?;
    }

    Ok(())
}

/// Checks if a firewall rule exists by name.
///
/// # Errors
/// Returns an error if the check fails.
pub fn rule_exists(name: &str) -> Result<bool> {
    let _ = DropUninitializeCom;
    initialize_com()?;

    let mut exists = false;

    unsafe {
        let fw_policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_INPROC_SERVER)?;
        let rules = fw_policy.Rules()?;
        let rule = rules.Item(&windows::core::BSTR::from(name));
        if rule.is_ok() {
            exists = true;
        }
    }

    Ok(exists)
}
