// BDD tests for Community domain: notices, skills, shared_objects, resource_bookings, gamification
// Step definitions will be implemented in Phase 4

use cucumber::{given, World};
use koprogo_api::application::ports::BuildingRepository;
use koprogo_api::application::use_cases::{
    AchievementUseCases, ChallengeUseCases, NoticeUseCases, ResourceBookingUseCases,
    SharedObjectUseCases, SkillUseCases,
};
use koprogo_api::infrastructure::database::{
    create_pool, PostgresAchievementRepository, PostgresBuildingRepository,
    PostgresChallengeProgressRepository, PostgresChallengeRepository, PostgresNoticeRepository,
    PostgresOwnerCreditBalanceRepository, PostgresOwnerRepository,
    PostgresResourceBookingRepository, PostgresSharedObjectRepository, PostgresSkillRepository,
    PostgresUserAchievementRepository, PostgresUserRepository,
};
use koprogo_api::infrastructure::pool::DbPool;
use std::sync::Arc;
use std::time::Duration;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};
use tokio::time::sleep;
use uuid::Uuid;

#[derive(World)]
#[world(init = Self::new)]
#[allow(dead_code)]
pub struct CommunityWorld {
    _container: Option<ContainerAsync<Postgres>>,
    pool: Option<DbPool>,
    org_id: Option<Uuid>,
    building_id: Option<Uuid>,

    notice_use_cases: Option<Arc<NoticeUseCases>>,
    skill_use_cases: Option<Arc<SkillUseCases>>,
    shared_object_use_cases: Option<Arc<SharedObjectUseCases>>,
    resource_booking_use_cases: Option<Arc<ResourceBookingUseCases>>,
    achievement_use_cases: Option<Arc<AchievementUseCases>>,
    challenge_use_cases: Option<Arc<ChallengeUseCases>>,

    last_result: Option<Result<String, String>>,
    last_notice_id: Option<Uuid>,
    last_owner_id: Option<Uuid>,
}

impl std::fmt::Debug for CommunityWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommunityWorld")
            .field("org_id", &self.org_id)
            .finish()
    }
}

impl CommunityWorld {
    async fn new() -> Self {
        Self {
            _container: None,
            pool: None,
            org_id: None,
            building_id: None,
            notice_use_cases: None,
            skill_use_cases: None,
            shared_object_use_cases: None,
            resource_booking_use_cases: None,
            achievement_use_cases: None,
            challenge_use_cases: None,
            last_result: None,
            last_notice_id: None,
            last_owner_id: None,
        }
    }

    async fn setup_database(&mut self) {
        let mut attempts = 0;
        let postgres_container = loop {
            match Postgres::default().start().await {
                Ok(container) => break container,
                Err(e) if attempts < 3 => {
                    attempts += 1;
                    sleep(Duration::from_millis(500)).await;
                }
                Err(e) => panic!("Failed to start postgres container: {}", e),
            }
        };

        let host_port = postgres_container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get host port");
        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );
        let pool = create_pool(&connection_string)
            .await
            .expect("Failed to create pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        self.pool = Some(pool.clone());

        let org_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO organizations (id, name, slug, contact_email, subscription_plan, max_buildings, max_users, is_active, created_at, updated_at)
               VALUES ($1, 'Community BDD Org', 'com-bdd', 'com@bdd.com', 'starter', 10, 10, true, NOW(), NOW())"#
        )
        .bind(org_id)
        .execute(&pool)
        .await
        .expect("insert org");

        let building_repo: Arc<dyn BuildingRepository> =
            Arc::new(PostgresBuildingRepository::new(pool.clone()));
        {
            use koprogo_api::domain::entities::Building;
            let b = Building::new(
                org_id,
                "Residence Communautaire".to_string(),
                "1 Place du Village".to_string(),
                "Namur".to_string(),
                "5000".to_string(),
                "Belgique".to_string(),
                10,
                1000,
                Some(2000),
            )
            .unwrap();
            building_repo.create(&b).await.expect("create building");
            self.building_id = Some(b.id);
        }

        let notice_repo = Arc::new(PostgresNoticeRepository::new(pool.clone()));
        let skill_repo = Arc::new(PostgresSkillRepository::new(pool.clone()));
        let shared_object_repo = Arc::new(PostgresSharedObjectRepository::new(pool.clone()));
        let booking_repo = Arc::new(PostgresResourceBookingRepository::new(pool.clone()));
        let owner_repo = Arc::new(PostgresOwnerRepository::new(pool.clone()));
        let credit_balance_repo = Arc::new(PostgresOwnerCreditBalanceRepository::new(pool.clone()));
        let achievement_repo = Arc::new(PostgresAchievementRepository::new(pool.clone()));
        let user_achievement_repo = Arc::new(PostgresUserAchievementRepository::new(pool.clone()));
        let challenge_repo = Arc::new(PostgresChallengeRepository::new(pool.clone()));
        let challenge_progress_repo =
            Arc::new(PostgresChallengeProgressRepository::new(pool.clone()));
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));

        let notice_use_cases = NoticeUseCases::new(notice_repo, owner_repo.clone());
        let skill_use_cases = SkillUseCases::new(skill_repo, owner_repo.clone());
        // SharedObjectUseCases: shared_object_repo + owner_repo + credit_balance_repo
        let shared_object_use_cases =
            SharedObjectUseCases::new(shared_object_repo, owner_repo.clone(), credit_balance_repo);
        let resource_booking_use_cases = ResourceBookingUseCases::new(booking_repo, owner_repo);
        // AchievementUseCases: achievement_repo + user_achievement_repo + user_repo
        let achievement_use_cases =
            AchievementUseCases::new(achievement_repo, user_achievement_repo, user_repo);
        let challenge_use_cases = ChallengeUseCases::new(challenge_repo, challenge_progress_repo);

        self.notice_use_cases = Some(Arc::new(notice_use_cases));
        self.skill_use_cases = Some(Arc::new(skill_use_cases));
        self.shared_object_use_cases = Some(Arc::new(shared_object_use_cases));
        self.resource_booking_use_cases = Some(Arc::new(resource_booking_use_cases));
        self.achievement_use_cases = Some(Arc::new(achievement_use_cases));
        self.challenge_use_cases = Some(Arc::new(challenge_use_cases));
        self._container = Some(postgres_container);
        self.org_id = Some(org_id);
    }
}

#[given("the system is initialized")]
async fn given_system_initialized(world: &mut CommunityWorld) {
    world.setup_database().await;
}

#[tokio::main]
async fn main() {
    CommunityWorld::cucumber()
        .run_and_exit("tests/features/notices.feature")
        .await;
}
