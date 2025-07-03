//! Certificate verification for secure canister communication

use serde::{Deserialize, Serialize};
use crate::error::Result;

/// Certificate data from ICP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// The certificate tree
    pub tree: CertificateTree,
    /// The signature
    pub signature: Vec<u8>,
    /// Optional delegation
    pub delegation: Option<Delegation>,
}

/// Certificate tree structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateTree {
    Empty,
    Fork(Box<CertificateTree>, Box<CertificateTree>),
    Labeled(Vec<u8>, Box<CertificateTree>),
    Leaf(Vec<u8>),
    Pruned(Vec<u8>),
}

/// Delegation for certificate chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub subnet_id: Vec<u8>,
    pub certificate: Vec<u8>,
}

/// Trait for certificate verification
pub trait CertificateVerifier: Send + Sync {
    /// Verify a certificate
    fn verify(&self, certificate: &Certificate) -> Result<bool>;
    
    /// Extract certified data
    fn extract_data(&self, certificate: &Certificate, path: &[&[u8]]) -> Result<Option<Vec<u8>>>;
}

/// Certificate verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateConfig {
    /// Whether to verify certificates
    pub verify_certificates: bool,
    /// Root public key for verification
    pub root_key: Option<Vec<u8>>,
    /// Maximum certificate age in seconds
    pub max_age_secs: u64,
}

impl Default for CertificateConfig {
    fn default() -> Self {
        Self {
            verify_certificates: true,
            root_key: None,
            max_age_secs: 300, // 5 minutes
        }
    }
}

/// Basic certificate verifier implementation
pub struct BasicCertificateVerifier {
    config: CertificateConfig,
}

impl BasicCertificateVerifier {
    /// Create a new certificate verifier
    pub fn new(config: CertificateConfig) -> Self {
        Self { config }
    }
    
    /// Look up a path in the certificate tree
    fn lookup_path(&self, tree: &CertificateTree, path: &[&[u8]]) -> Option<Vec<u8>> {
        match tree {
            _ if path.is_empty() => match tree {
                CertificateTree::Leaf(data) => Some(data.clone()),
                _ => None,
            },
            CertificateTree::Fork(left, right) => {
                self.lookup_path(left, path)
                    .or_else(|| self.lookup_path(right, path))
            }
            CertificateTree::Labeled(label, subtree) if !path.is_empty() => {
                if label == path[0] {
                    self.lookup_path(subtree, &path[1..])
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl CertificateVerifier for BasicCertificateVerifier {
    fn verify(&self, _certificate: &Certificate) -> Result<bool> {
        if !self.config.verify_certificates {
            return Ok(true);
        }
        
        // TODO: Implement actual certificate verification
        // This requires cryptographic operations and ICP-specific logic
        // For now, we'll just return true if verification is disabled
        Ok(false)
    }
    
    fn extract_data(&self, certificate: &Certificate, path: &[&[u8]]) -> Result<Option<Vec<u8>>> {
        Ok(self.lookup_path(&certificate.tree, path))
    }
}

/// Helper for building certificate paths
pub struct CertificatePath {
    segments: Vec<Vec<u8>>,
}

impl CertificatePath {
    /// Create a new certificate path
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
    
    /// Add a segment to the path
    pub fn segment(mut self, segment: impl AsRef<[u8]>) -> Self {
        self.segments.push(segment.as_ref().to_vec());
        self
    }
    
    /// Add a string segment
    pub fn string_segment(self, segment: &str) -> Self {
        self.segment(segment.as_bytes())
    }
    
    /// Build the path segments
    pub fn build(self) -> Vec<Vec<u8>> {
        self.segments
    }
}

/// Response with certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertifiedResponse<T> {
    /// The response data
    pub data: T,
    /// The certificate
    pub certificate: Certificate,
}

impl<T> CertifiedResponse<T> {
    /// Create a new certified response
    pub fn new(data: T, certificate: Certificate) -> Self {
        Self { data, certificate }
    }
    
    /// Verify the certificate
    pub fn verify(&self, verifier: &dyn CertificateVerifier) -> Result<bool> {
        verifier.verify(&self.certificate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_certificate_path_builder() {
        let path = CertificatePath::new()
            .string_segment("canister")
            .segment([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01])
            .string_segment("certified_data")
            .build();
            
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], b"canister");
        assert_eq!(path[2], b"certified_data");
    }
}