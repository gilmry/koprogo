use cucumber::{given, then, when, World};
use koprogo_api::application::dto::CreateBuildingDto;
use koprogo_api::application::use_cases::BuildingUseCases;
use koprogo_api::infrastructure::database::{create_pool, PostgresBuildingRepository};
use std::sync::Arc;
use testcontainers::clients::Cli;
use testcontainers_modules::postgres::Postgres;

#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct BuildingWorld {
    docker: Option<Cli>,
    postgres_container: Option<testcontainers::Container<'static, Postgres>>,
    use_cases: Option<Arc<BuildingUseCases>>,
    building_dto: Option<CreateBuildingDto>,
    last_result: Option<Result<String, String>>,
}

impl BuildingWorld {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            docker: None,
            postgres_container: None,
            use_cases: None,
            building_dto: None,
            last_result: None,
        })
    }

    async fn setup_database(&mut self) {
        let docker = Cli::default();
        let postgres_container = docker.run(Postgres::default());
        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres_container.get_host_port_ipv4(5432)
        );

        let pool = create_pool(&connection_string)
            .await
            .expect("Failed to create pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        let repo = Arc::new(PostgresBuildingRepository::new(pool));
        let use_cases = BuildingUseCases::new(repo);

        self.docker = Some(docker);
        self.postgres_container = Some(postgres_container);
        self.use_cases = Some(Arc::new(use_cases));
    }
}

#[given("a coproperty management system")]
async fn given_system(world: &mut BuildingWorld) {
    world.setup_database().await;
}

#[when(regex = r#"^I create a building named "([^"]*)" in "([^"]*)"$"#)]
async fn when_create_building(world: &mut BuildingWorld, name: String, city: String) {
    let dto = CreateBuildingDto {
        name: name.clone(),
        address: "123 Test St".to_string(),
        city: city.clone(),
        postal_code: "75001".to_string(),
        country: "France".to_string(),
        total_units: 10,
        construction_year: Some(2000),
    };

    world.building_dto = Some(dto.clone());

    if let Some(use_cases) = &world.use_cases {
        let result = use_cases.create_building(dto).await;
        world.last_result = Some(result.map(|b| b.id).map_err(|e| e.to_string()));
    }
}

#[then("the building should be created successfully")]
async fn then_building_created(world: &mut BuildingWorld) {
    assert!(world.last_result.is_some());
    assert!(world.last_result.as_ref().unwrap().is_ok());
}

#[then(regex = r#"^the building should be in "([^"]*)"$"#)]
async fn then_building_in_city(world: &mut BuildingWorld, city: String) {
    assert!(world.building_dto.is_some());
    assert_eq!(world.building_dto.as_ref().unwrap().city, city);
}

#[tokio::main]
async fn main() {
    BuildingWorld::cucumber()
        .run_and_exit("tests/features/building.feature")
        .await;
}
