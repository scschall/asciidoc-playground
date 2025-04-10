use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct SecurityNamespaceResponse {
    pub value: Vec<SecurityNamespace>
}

#[derive(Debug, Deserialize)]
pub struct SecurityNamespace {
    pub namespaceId: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct GroupsResponse {
    pub value: Vec<GroupInfo>
}

#[derive(Debug, Deserialize)]
pub struct GroupInfo {
    pub displayName: String,
    pub description: Option<String>,
    pub descriptor: String,
    pub originId: String,
}

#[derive(Debug, Deserialize)]
pub struct MembershipResponse {
    pub value: Vec<Membership>
}

#[derive(Debug, Deserialize)]
pub struct Membership {
    pub memberDescriptor: String,
}

#[derive(Debug, Deserialize)]
pub struct UserResponse {
    pub displayName: String,
    pub mailAddress: Option<String>,
}

pub struct AzureDevOpsClient {
    client: reqwest::Client,
    organization: String,
    auth: String,
}

impl AzureDevOpsClient {
    pub fn new(organization: String, pat: String) -> Self {
        let client = reqwest::Client::new();
        let auth = format!("Basic {}", base64::encode(format!(":{}", pat)));
        
        Self {
            client,
            organization,
            auth,
        }
    }

    pub async fn get_security_namespaces(&self) -> Result<Vec<SecurityNamespace>, Box<dyn Error>> {
        let url = format!(
            "https://dev.azure.com/{}/_apis/securitynamespaces?api-version=7.1-preview.1",
            self.organization
        );

        let response_text = self.client
            .get(&url)
            .header("Authorization", &self.auth)
            .send()
            .await?
            .text()
            .await?;

        let response: SecurityNamespaceResponse = serde_json::from_str(&response_text)?;
        Ok(response.value)
    }

    pub async fn get_groups(&self) -> Result<Vec<GroupInfo>, Box<dyn Error>> {
        let url = format!(
            "https://vssps.dev.azure.com/{}/_apis/graph/groups?api-version=5.1-preview.1",
            self.organization
        );

        let response_text = self.client
            .get(&url)
            .header("Authorization", &self.auth)
            .send()
            .await?
            .text()
            .await?;

        let response: GroupsResponse = serde_json::from_str(&response_text)?;
        Ok(response.value)
    }

    pub async fn get_group_members(&self, group_descriptor: &str) -> Result<Vec<Membership>, Box<dyn Error>> {
        let url = format!(
            "https://vssps.dev.azure.com/{}/_apis/graph/memberships/{}?direction=down&api-version=5.1-preview.1",
            self.organization,
            group_descriptor
        );

        let response_text = self.client
            .get(&url)
            .header("Authorization", &self.auth)
            .send()
            .await?
            .text()
            .await?;

        let response: MembershipResponse = serde_json::from_str(&response_text)?;
        Ok(response.value)
    }

    pub async fn get_user(&self, user_descriptor: &str) -> Result<UserResponse, Box<dyn Error>> {
        let url = format!(
            "https://vssps.dev.azure.com/{}/_apis/graph/users/{}?api-version=5.1-preview.1",
            self.organization,
            user_descriptor
        );

        let response_text = self.client
            .get(&url)
            .header("Authorization", &self.auth)
            .send()
            .await?
            .text()
            .await?;

        let user: UserResponse = serde_json::from_str(&response_text)?;
        Ok(user)
    }
} 