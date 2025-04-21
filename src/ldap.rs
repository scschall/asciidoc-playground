use ldap3::{LdapConn, Scope, SearchEntry};
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct LdapGroup {
    pub name: String,
    pub description: Option<String>,
    pub owner: Option<String>,
}

pub struct LdapClient {
    conn: LdapConn,
    base_dn: String,
}

impl LdapClient {
    pub fn new(
        ldap_url: &str,
        bind_dn: &str,
        bind_pw: &str,
        base_dn: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let mut conn = LdapConn::new(ldap_url)?;
        conn.simple_bind(bind_dn, bind_pw)?;

        Ok(Self {
            conn,
            base_dn: base_dn.to_string(),
        })
    }

    pub fn get_group(&mut self, group_name: &str) -> Result<Option<LdapGroup>, Box<dyn Error>> {
        let filter = format!("(&(objectClass=group)(cn={}))", group_name);
        let attrs = vec!["cn", "description", "managedBy"];

        let result = self
            .conn
            .search(&self.base_dn, Scope::Subtree, &filter, attrs)?;

        if let Some(entry) = result.into_iter().next() {
            let entry = SearchEntry::construct(entry);

            Ok(Some(LdapGroup {
                name: entry
                    .attrs
                    .get("cn")
                    .and_then(|v| v.first())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                description: entry
                    .attrs
                    .get("description")
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
                owner: entry
                    .attrs
                    .get("managedBy")
                    .and_then(|v| v.first())
                    .map(|s| s.to_string()),
            }))
        } else {
            Ok(None)
        }
    }
}
