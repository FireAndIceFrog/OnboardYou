/// A resolved routing entry from the `EmailRoutes` DynamoDB table.
#[derive(Debug, Clone, PartialEq)]
pub struct EmailRoute {
    pub org_id: String,
    pub company_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_route_fields_are_accessible() {
        let route = EmailRoute {
            org_id: "org-1".into(),
            company_id: "company-1".into(),
        };
        assert_eq!(route.org_id, "org-1");
        assert_eq!(route.company_id, "company-1");
    }
}
