use async_trait;
use node_template_runtime::{inherents_example::INHERENT_IDENTIFIER, InherentDataType};
use sp_core::Encode;
use sp_inherents::{InherentData, InherentIdentifier};
use std::fmt::Debug;

/// The provider of inherent data from the Node to the Runtime
/// It holds data of InherentDataType defined by the Runtime
#[derive(Debug, Clone)]
pub struct ExternalDataInherentProvider(pub Option<InherentDataType>);

/// Implementation of sp_inherents::InherentDataProvider trait for ExternalDataInherentProvider
#[async_trait::async_trait]
impl sp_inherents::InherentDataProvider for ExternalDataInherentProvider {
	async fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		if let Some(ref data) = self.0 {
			inherent_data.put_data(INHERENT_IDENTIFIER, &data.encode())
		} else {
			Ok(())
		}
	}

	async fn try_handle_error(
		&self,
		identifier: &InherentIdentifier,
		error: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		// handle only data identified by INHERENT_IDENTIFIER key, ignore the rest
		if *identifier == INHERENT_IDENTIFIER {
			Some(Err(sp_inherents::Error::Application(Box::from(std::format!(
				"Error processing inherent: {:?}",
				error
			)))))
		} else {
			None
		}
	}
}
