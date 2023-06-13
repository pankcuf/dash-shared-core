pub mod core {
    use rs_dapi_client::{DAPIClient, GetStatusRequest};
    use rs_dapi_client::rs_dapi_client::GetStatusResponse;

    pub async fn get_status(host: &'static str) -> tonic::Response<GetStatusResponse> {
        // let addr = "http://34.213.25.113:3010";
        let mut service = DAPIClient::new(host)
            .core_service();
        let response = service.get_status(GetStatusRequest {})
            .await
            .expect("Expect response");
        response
    }
}
