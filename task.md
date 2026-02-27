# Simple pipeline inference from initial api -> output

Our goal: Create a simple api that can create a custom pipeline using AI inference at scale. 

This has to be async, triggering an sqs queue that pulls the openapi schema and then stores the fields nessicary for the schema. 

This schema generation is agentic in nature; we try parsing a working json with errors attached in runtime and we send that to the agent so it can resolve them.

# Functional: Creating + saving the output schema of a default pipeline.
0. when settings credentials are saved, send a message to trigger this lambda. 
1. We save an org settings. We return a status for schema_generation_status
2. When we get settings, schema_generation_status is there. 

3. create an agentic loop. We send an object with the information {url: string; openapi_schema?: string} (schema is for testing)
4. It pulls the url's json into there if its not already
5. we create a json object called the output {field_name: type(str default)}
6. final object is {output_type: {field_name: str}, output_object_path: str, output_api_template: obj}
6. We compile it. if it fails, we tell the agent the issue & make it repeat until it works.

NOTES:
the api endpoint should always contain a list<targetType> where targetType is our expected type. This object can be N nested so we should store the PATH only.

The output_api_template is the template with hardcoded fields.
The output_object_path is the path in the template we are overriding
The output type is the dynamic type we need to map to. THis is always a list containing our type.

# Non functional requirements
Use dependancy injection the same way we have done it for the rest of the api's
```
use std::sync::Arc;

use crate::repositories::{
    config_repository::{self, DynamoConfigRepo},
    etl_repository::{EtlRepository, IEtlRepo},
    pipeline_repository::{IPipelineRepo, PipelineRepository},
    settings_repository::{self, DynamoSettingsRepo},
};
use config_repository::IConfigRepo;
use onboard_you::ActionFactoryTrait;
use settings_repository::ISettingsRepo;

/// Environment configuration read from process env.
#[derive(Clone, Default)]
pub struct Env {
    pub table_name: String,
    pub settings_table_name: String,
}

impl Env {
    pub fn from_env() -> Arc<Self> {
        Arc::new(Self {
            table_name: std::env::var("CONFIG_TABLE_NAME")
                .unwrap_or_else(|_| "PipelineConfigs".to_string()),
            settings_table_name: std::env::var("SETTINGS_TABLE_NAME")
                .unwrap_or_else(|_| "OrgSettings".to_string()),
        })
    }
}

// Traits and concrete implementations live in the repository modules.

/// Runtime dependancies (repositories/engines) constructed from `Env`.
pub struct Dependancies {
    pub config_repo: Arc<dyn IConfigRepo>,
    pub settings_repo: Arc<dyn ISettingsRepo>,
    pub etl_repo: Arc<dyn IEtlRepo>,
    pub pipeline_repo: Arc<dyn IPipelineRepo>,
    pub action_factory: Arc<dyn ActionFactoryTrait>,
}

impl Dependancies {
    /// Create a new `Dependancies` from the provided `Env`.
    /// This loads the AWS config and constructs the clients/repositories, so it's async.
    pub async fn new(cfg: Arc<Env>) -> Self {
        let aws_config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
        let dynamo = aws_sdk_dynamodb::Client::new(&aws_config);

        // Construct concrete Dynamo-backed repo implementations from their modules

        Self {
            config_repo: DynamoConfigRepo::new(dynamo.clone(), cfg.table_name.clone()),
            settings_repo: DynamoSettingsRepo::new(dynamo.clone(), cfg.settings_table_name.clone()),
            etl_repo: EtlRepository::new(),
            pipeline_repo: PipelineRepository::new(),
            action_factory: Arc::new(onboard_you::ActionFactory::new()),
        }
    }
}
```

```gh_models example

use gh_models::{GHModels, types::ChatMessage};
use std::env;

#[tokio::main]
async fn main() {
    let token = env::var("GITHUB_TOKEN").expect("Missing GITHUB_TOKEN");
    let client = GHModels::new(token);

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a helpful assistant.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: "What is the capital of France?".into(),
        },
    ];

    let response = client
        .chat_completion("openai/gpt-4o", &messages, 1.0, 4096, 1.0)
        .await
        .unwrap();

    println!("{}", response.choices[0].message.content);
}
```

# Use the engine -> repository pattern

Engines define HOW we deal with information and repositories handle WHERE we get information from 

We will need a repository for:
1. github access (gh_models crate - runs the agentic loop, contains the conversation points. Takes a string & responds with the next ai response)
2. Settings dynamodb access (see lambdas/api/src/repositories/settings_repository.rs)
4. openapi repo - constructs the openapi json schema (stored as string) from the url given

We will need an engine:
1. custom-api-coordinator (fetch settings, get schema, run model until its validated, save settings with status successful) 

# infra
Create new lambda with settings access + an event trigger from the save of settings if the credential changed