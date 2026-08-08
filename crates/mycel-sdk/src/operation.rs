/// Returns a new UUID v4 string suitable for
/// `mycel.client.v1.BeginTransactionRequest.operation_id`.
///
/// Operation IDs are client-side correlation metadata only. They are not
/// idempotency keys, authorization credentials, replay protection, or commit
/// ordering guarantees.
pub fn new_operation_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_operation_id_returns_uuid_v4() {
        let operation_id = new_operation_id();
        let parsed = uuid::Uuid::parse_str(&operation_id).expect("operation ID should parse");
        assert_eq!(parsed.get_version_num(), 4);
        assert_eq!(operation_id.len(), 36);
    }
}
