mod azure_devops;

use azure_devops::AzureDevOpsClient;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let organization = env::var("AZURE_DEVOPS_ORG")?;
    let pat = env::var("AZURE_DEVOPS_PAT")?;

    let client = AzureDevOpsClient::new(organization, pat);

    // Get all groups
    let groups = client.get_groups().await?;
    
    // Find Project Administrators group
    let project_admin_group = groups.iter()
        .find(|g| g.displayName.ends_with("Project Administrators"))
        .expect("Project Administrators group not found");

    println!("Project Administrators group: {}", project_admin_group.displayName);

    // Get group members
    let members = client.get_group_members(&project_admin_group.descriptor).await?;

    // Get user info for each member
    for member in members {
        let user = client.get_user(&member.memberDescriptor).await?;
        println!("User: {} ({})", 
            user.displayName,
            user.mailAddress.unwrap_or_else(|| "No email".to_string())
        );
    }

    Ok(())
}
