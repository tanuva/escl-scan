pub struct Scanner {
    base_url: String,
}

impl Scanner {
    pub fn new(ip_or_host: String, root: Option<String>) -> Scanner {
        let resource_root = if root.is_some() {
            root.unwrap()
        } else {
            "eSCL".to_string()
        };

        Scanner {
            base_url: format!("http://{}:80/{}", ip_or_host, resource_root),
        }
    }
}
