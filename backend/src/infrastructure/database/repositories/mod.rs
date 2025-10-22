pub mod building_repository_impl;
pub mod document_repository_impl;
pub mod expense_repository_impl;
pub mod meeting_repository_impl;
pub mod owner_repository_impl;
pub mod unit_repository_impl;

pub use building_repository_impl::PostgresBuildingRepository;
pub use document_repository_impl::PostgresDocumentRepository;
pub use expense_repository_impl::PostgresExpenseRepository;
pub use meeting_repository_impl::PostgresMeetingRepository;
pub use owner_repository_impl::PostgresOwnerRepository;
pub use unit_repository_impl::PostgresUnitRepository;
