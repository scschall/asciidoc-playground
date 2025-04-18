mod azure_devops;

use azure_devops::AzureDevOpsClient;
use std::collections::HashMap;
use std::{env, fs};

async fn update_admins_in_file(
    content: &str,
    client: &AzureDevOpsClient,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut variables = HashMap::new();
    let mut updated_content = String::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("//little-doc-helper remember ") {
            let var_name = line
                .trim_start_matches("//little-doc-helper remember ")
                .trim();
            if let Some(next_line) = lines.next() {
                variables.insert(
                    var_name.to_string(),
                    next_line
                        .trim_start_matches("== ")
                        .trim_start_matches("=== ")
                        .to_string(),
                );
                updated_content.push_str(line);
                updated_content.push('\n');
                updated_content.push_str(next_line);
                updated_content.push('\n');
            }
        } else if line.starts_with("//little-doc-helper maintain azure-devops-project-admins") {
            let organization = variables
                .get("azure-organization")
                .expect("azure-organization not found");
            let project = variables
                .get("azure-project")
                .expect("azure-project not found");

            // Get admins from Azure DevOps
            let groups = client.get_groups(organization).await?;

            let project_admin_groups: Vec<_> = groups
                .iter()
                .filter(|g| g.principalName == format!("[{}]\\Project Administrators", project))
                .collect();

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
            let mut admins = Vec::new();
            for group in project_admin_groups {
                let members = client
                    .get_group_members(organization, &group.descriptor)
                    .await?;
                for member in members {
                    let user = client
                        .get_user(organization, &member.memberDescriptor)
                        .await?;
                    admins.push((user.displayName, user.mailAddress));
                }
            }

            // Sort admins by display name
            admins.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

            // Add sorted list to output
            for (name, email) in admins {
                updated_content.push_str(&format!(
                    ". {} ({})\n",
                    name,
                    email.unwrap_or("".to_string())
                ));
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

    // write updated content to file
    fs::write("index.adoc", updated_content)?;
    Ok(())
}
