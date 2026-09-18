use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfImportRequest {
    pub url: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfImportPlan {
    pub source_url: String,
    pub host: String,
    pub mode: String,
    pub requires_login: bool,
    pub network_execution: bool,
    pub next_steps: Vec<String>,
    pub policy: String,
}

pub fn plan_import(request: CtfImportRequest) -> Result<CtfImportPlan, String> {
    let parsed = Url::parse(&request.url).map_err(|e| format!("invalid URL: {e}"))?;
    let host = parsed
        .host_str()
        .ok_or("URL must include a host")?
        .to_ascii_lowercase();
    if parsed.scheme() != "https"
        && !(parsed.scheme() == "http" && (host == "localhost" || host == "127.0.0.1"))
    {
        return Err("CTF sources require HTTPS, except local test fixtures".into());
    }
    let (mode, requires_login) = if host == "github.com" || host.ends_with(".github.com") {
        ("repository_metadata", false)
    } else if host == "app.hackthebox.com" || host.ends_with("hackthebox.com") {
        ("challenge_reference", true)
    } else if host == "localhost" || host == "127.0.0.1" {
        ("local_fixture", false)
    } else {
        return Err("host is not allowlisted; use GitHub, Hack The Box, or a local fixture".into());
    };
    Ok(CtfImportPlan {
        source_url: request.url,
        host,
        mode: mode.into(),
        requires_login,
        network_execution: false,
        next_steps: vec![
            "Review license and authorization".into(),
            "Pin an immutable revision or challenge identifier".into(),
            "Materialize into a disposable workspace".into(),
            "Run only an isolated local worker after human approval".into(),
        ],
        policy: "metadata-only intake; no remote execution, scanning, exploit delivery, or credential handling".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn plans_github_without_execution() {
        let plan = plan_import(CtfImportRequest {
            url: "https://github.com/example/ctf".into(),
            name: None,
        })
        .unwrap();
        assert_eq!(plan.mode, "repository_metadata");
        assert!(!plan.network_execution);
    }
    #[test]
    fn requires_login_for_htb() {
        let plan = plan_import(CtfImportRequest {
            url: "https://app.hackthebox.com/challenges/1".into(),
            name: None,
        })
        .unwrap();
        assert!(plan.requires_login);
    }
    #[test]
    fn rejects_unknown_host() {
        assert!(plan_import(CtfImportRequest {
            url: "https://random.example/ctf".into(),
            name: None
        })
        .is_err());
    }
}
