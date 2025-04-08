use reqwest;
use serde::Deserialize;
use std::env;
use serde_json;

#[derive(Debug, Deserialize)]
struct User {
    displayName: String,
    mailAddress: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SecurityNamespace {
    namespaceId: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct AccessControlEntry {
    descriptor: String,
    allow: i32,
}

#[derive(Debug, Deserialize)]
struct SecurityNamespaceResponse {
    value: Vec<SecurityNamespace>
}

#[derive(Debug, Deserialize)]
struct AccessControlList {
    acesDictionary: std::collections::HashMap<String, AccessControlEntry>,
}

#[derive(Debug, Deserialize)]
struct AccessControlListResponse {
    value: Vec<AccessControlList>
}

#[derive(Debug, Deserialize)]
struct GroupsResponse {
    value: Vec<GroupInfo>
}

#[derive(Debug, Deserialize)]
struct GroupInfo {
    displayName: String,
    description: Option<String>,
    descriptor: String,
    originId: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let organization = env::var("AZURE_DEVOPS_ORG")?;
    let project = env::var("AZURE_DEVOPS_PROJECT")?;
    let pat = env::var("AZURE_DEVOPS_PAT")?;

    let client = reqwest::Client::new();
    let auth = format!("Basic {}", base64::encode(format!(":{}", pat)));

    // Get security namespaces
    let url = format!(
        "https://dev.azure.com/{}/_apis/securitynamespaces?api-version=7.1-preview.1",
        organization
    );

    println!("Getting security namespaces");
    let response_text = client
        .get(&url)
        .header("Authorization", &auth)
        .send()
        .await?
        .text()
        .await?;
    println!("Raw response: {}", response_text);

    let response: SecurityNamespaceResponse = serde_json::from_str(&response_text)?;
    println!("Security namespaces: {:?}", response.value);
    let project_namespace = response.value
        .iter()
        .find(|n| n.name == "Project")
        .expect("Project namespace not found");

    println!("Project namespace: {:?}", project_namespace);
    // Get ACLs
    let project_token = format!("$PROJECT:vstfs:///Classification/TeamProject/{}", project);
    let url = format!(
        "https://dev.azure.com/{}/_apis/accesscontrollists/{}?token={}&includeExtendedInfo=true&api-version=7.1-preview.1",
        organization, project_namespace.namespaceId, project_token
    );

    let response_text = client
        .get(&url)
        .header("Authorization", &auth)
        .send()
        .await?
        .text()
        .await?;
    println!("ACLs response: {}", response_text);

    let response: AccessControlListResponse = serde_json::from_str(&response_text)?;
    let acls = response.value;

    // Get users
    for acl in acls {
        for (descriptor, ace) in acl.acesDictionary {
            if (ace.allow & 4) != 0 {
                let url = format!(
                    "https://vssps.dev.azure.com/{}/_apis/graph/users/{}",
                    organization, descriptor
                );

                let user: User = client
                    .get(&url)
                    .header("Authorization", &auth)
                    .send()
                    .await?
                    .json()
                    .await?;

                println!("Administrator: {} ({})",
                    user.displayName,
                    user.mailAddress.unwrap_or_else(|| "No email".to_string()));
            }
        }
    }

    // Get all groups
    let url = format!(
        "https://vssps.dev.azure.com/{}/_apis/graph/groups?api-version=5.1-preview.1",
        organization
    );

    let response_text = client
    .get(&url)
    .header("Authorization", &auth)
    .send()
    .await?
    .text()
    .await?;
    println!("Groups response: {}", response_text);

    let response: GroupsResponse = serde_json::from_str(&response_text)?;
    
    println!("\nGefundene Gruppen:");
    for group in &response.value {
        println!("- {} ({:?})", 
            group.displayName, 
            group.description
        );
    }

    // find group with displayName "Project Administrators"
    let project_admin_group = response.value
        .iter()
        .find(|g| g.displayName == "[ORGANIZATION]\\Project Administrators")
        .or_else(|| response.value.iter().find(|g| g.displayName.ends_with("Project Administrators")))
        .expect("Project Administrators group not found");

    println!("Project Administrators group: {}\n {:?})", 
        project_admin_group.displayName,
        project_admin_group);

    // List all members of the project administrators group
    let url = format!(
        "https://vssps.dev.azure.com/{}/_apis/graph/memberships/{}?direction=down&api-version=5.1-preview.1",
        organization,
        project_admin_group.descriptor
    );

    println!("Group memberships URL: {}", url);

    let response_text = client
        .get(&url)
        .header("Authorization", &auth)
        .send()
        .await?
        .text()
        .await?;
    println!("Group memberships response: {}", response_text);

    Ok(())
}
