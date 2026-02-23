use onboard_you::ActionFactory;

pub struct EtlRepository {
    // Define the structure of your ETL repository here
}

pub trait EtlRepo: Send + Sync {
    fn create_action_factory(&self) -> ActionFactory;
}

impl EtlRepo for EtlRepository {
    fn create_action_factory(&self) -> ActionFactory {
        // Implement the logic to create and return an ActionFactory instance
        ActionFactory::new()
    }
}
