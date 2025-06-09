use jp_postal_address::{
    postal_address_service_server::PostalAddressService, search_postal_address_response,
    PostalAddress, SearchPostalAddressRequest, SearchPostalAddressResponse,
};
use tonic::{Request, Response, Status};

use crate::{infra, usecase};

#[derive(Debug)]
pub struct PostalAddressServiceImpl {
    repo: infra::postgres::UtfKenAllRepositoryPostgres,
}

impl PostalAddressServiceImpl {
    pub fn new(repo: infra::postgres::UtfKenAllRepositoryPostgres) -> Self {
        Self { repo }
    }
}

#[tonic::async_trait]
impl PostalAddressService for PostalAddressServiceImpl {
    async fn search_postal_address(
        &self,
        request: Request<SearchPostalAddressRequest>,
    ) -> Result<Response<SearchPostalAddressResponse>, Status> {
        let req = request.into_inner();

        tracing::info!(?req, "Received gRPC search postal address request");

        let response = usecase::search_postal_code(
            &self.repo,
            usecase::SearchPostalCodeRequest {
                postal_code: req.postal_code,
                page_size: req.page_size.map(|size| size as usize),
                page_token: req.page_token,
            },
        )
        .await
        .map_err(|e| {
            tracing::error!(?e, "Failed to search postal address via gRPC");
            Status::internal("Failed to search postal address")
        })?;

        let items = response
            .records
            .into_iter()
            .map(|r| search_postal_address_response::Item {
                address: Some(PostalAddress {
                    postal_code: r.postal_code,
                    prefecture: r.prefecture,
                    city: r.city,
                    town: r.town,
                    prefecture_kana: r.prefecture_kana,
                    city_kana: r.city_kana,
                    town_kana: r.town_kana,
                }),
            })
            .collect();

        Ok(Response::new(SearchPostalAddressResponse {
            items,
            next_page_token: response.next_page_token,
        }))
    }
}
