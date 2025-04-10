mod azure_devops;

use azure_devops::AzureDevOpsClient;
use std::{env, fs};
use std::collections::HashMap;

async fn update_admins_in_file(content: &str, client: &AzureDevOpsClient) -> Result<String, Box<dyn std::error::Error>> {
    let mut variables = HashMap::new();
    let mut updated_content = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("//little-doc-helper remember ") {
            let var_name = line.trim_start_matches("//little-doc-helper remember ").trim();
            if let Some(next_line) = lines.next() {
                variables.insert(var_name.to_string(), next_line.trim_start_matches("== ").trim_start_matches("=== ").to_string());
                updated_content.push_str(line);
                updated_content.push('\n');
                updated_content.push_str(next_line);
                updated_content.push('\n');
            }
        } else if line.starts_with("//little-doc-helper maintain azure-devops-project-admins") {
            let organization = variables.get("azure-organization").expect("azure-organization not found");
            let project = variables.get("azure-project").expect("azure-project not found");

            // Get admins from Azure DevOps
            let groups = client.get_groups(organization).await?;
            println!("Found groups: {:?}", groups);
            
            let project_admin_groups: Vec<_> = groups.iter()
                .filter(|g| {
                    g.displayName.ends_with("Project Administrators") || 
                    g.displayName.contains(&format!("{} Project Administrators", project))
                })
                .collect();

            println!("Found admin groups: {:?}", project_admin_groups);

            if project_admin_groups.is_empty() {
                println!("Warning: No Project Administrator groups found!");
            }

            // Add the comment line
            updated_content.push_str(line);
            updated_content.push('\n');

            // Skip existing list
            while let Some(next_line) = lines.peek() {
                if next_line.trim_start().starts_with('.') {
                    lines.next();
                } else {
                    break;
                }
            }

            // Add new admin list from all groups
            for group in project_admin_groups {
                let members = client.get_group_members(organization, &group.descriptor).await?;
                for member in members {
                    let user = client.get_user(organization, &member.memberDescriptor).await?;
                    updated_content.push_str(&format!(". {}\n", user.displayName));
                }
            }
        } else {
            updated_content.push_str(line);
            updated_content.push('\n');
        }
    }

    Ok(updated_content)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pat = env::var("AZURE_DEVOPS_PAT")?;
    let client = AzureDevOpsClient::new(pat);
    
    let content = fs::read_to_string("index.adoc")?;
    let updated_content = update_admins_in_file(&content, &client).await?;
    
    println!("{}", updated_content);
    Ok(())
}
