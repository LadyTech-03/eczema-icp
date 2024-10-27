use candid::{CandidType, Deserialize, Principal};
use serde::Serialize;
use std::cell::RefCell;
use std::collections::{HashMap, BTreeMap};
use std::time::{SystemTime, UNIX_EPOCH};
use ic_cdk::storage;

const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 1000;
const PAGE_SIZE: usize = 20;

#[derive(CandidType, Serialize, Deserialize, Clone)]
pub struct EczemaResource {
    id: u64,
    title: String,
    description: String,
    category: ResourceCategory,
    created_at: u64,
    updated_at: u64,
    verified: bool,
    created_by: Principal,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceCategory {
    Treatment,
    Prevention,
    Research,
    DietAdvice,
    Testimonial,
    MedicalAdvice,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct CreateResourcePayload {
    title: String,
    description: String,
    category: ResourceCategory,
}

#[derive(CandidType, Serialize, Deserialize)]
pub enum EczemaError {
    NotFound,
    AlreadyExists,
    InvalidInput(String),
    Unauthorized,
    InternalError,
}

type EczemaResult<T> = Result<T, EczemaError>;

thread_local! {
    static ECZEMA_RESOURCES: RefCell<HashMap<u64, EczemaResource>> = RefCell::new(HashMap::new());
    static CATEGORY_INDEX: RefCell<BTreeMap<ResourceCategory, Vec<u64>>> = RefCell::new(BTreeMap::new());
    static NEXT_ID: RefCell<u64> = RefCell::new(1);
    static ADMINS: RefCell<Vec<Principal>> = RefCell::new(Vec::new());
}

fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn validate_payload(payload: &CreateResourcePayload) -> EczemaResult<()> {
    if payload.title.is_empty() || payload.title.len() > MAX_TITLE_LENGTH {
        return Err(EczemaError::InvalidInput("Invalid title length".to_string()));
    }
    if payload.description.is_empty() || payload.description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(EczemaError::InvalidInput("Invalid description length".to_string()));
    }
    Ok(())
}

fn is_admin(caller: Principal) -> bool {
    ADMINS.with(|admins| admins.borrow().contains(&caller))
}

#[ic_cdk_macros::update]
fn create_resource(payload: CreateResourcePayload) -> EczemaResult<EczemaResource> {
    validate_payload(&payload)?;
    let caller = ic_cdk::caller();

    NEXT_ID.with(|next_id| {
        ECZEMA_RESOURCES.with(|resources| {
            CATEGORY_INDEX.with(|category_index| {
                let id = *next_id.borrow();
                let timestamp = get_timestamp();

                let resource = EczemaResource {
                    id,
                    title: payload.title,
                    description: payload.description,
                    category: payload.category,
                    created_at: timestamp,
                    updated_at: timestamp,
                    verified: false,
                    created_by: caller,
                };

                resources.borrow_mut().insert(id, resource.clone());
                category_index.borrow_mut().entry(payload.category).or_default().push(id);
                *next_id.borrow_mut() += 1;
                Ok(resource)
            })
        })
    })
}

#[ic_cdk_macros::query]
fn get_resource(id: u64) -> EczemaResult<EczemaResource> {
    ECZEMA_RESOURCES.with(|resources| {
        resources
            .borrow()
            .get(&id)
            .cloned()
            .ok_or(EczemaError::NotFound)
    })
}

#[ic_cdk_macros::query]
fn list_resources(page: usize) -> Vec<EczemaResource> {
    ECZEMA_RESOURCES.with(|resources| {
        resources
            .borrow()
            .values()
            .skip(page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .cloned()
            .collect()
    })
}

#[ic_cdk_macros::query]
fn list_resources_by_category(category: ResourceCategory, page: usize) -> Vec<EczemaResource> {
    CATEGORY_INDEX.with(|category_index| {
        ECZEMA_RESOURCES.with(|resources| {
            category_index
                .borrow()
                .get(&category)
                .map(|ids| {
                    ids.iter()
                        .skip(page * PAGE_SIZE)
                        .take(PAGE_SIZE)
                        .filter_map(|id| resources.borrow().get(id).cloned())
                        .collect()
                })
                .unwrap_or_default()
        })
    })
}

#[ic_cdk_macros::update]
fn update_resource(id: u64, payload: CreateResourcePayload) -> EczemaResult<EczemaResource> {
    validate_payload(&payload)?;
    let caller = ic_cdk::caller();

    ECZEMA_RESOURCES.with(|resources| {
        let mut resources = resources.borrow_mut();
        if let Some(resource) = resources.get_mut(&id) {
            if resource.created_by != caller && !is_admin(caller) {
                return Err(EczemaError::Unauthorized);
            }
            resource.title = payload.title;
            resource.description = payload.description;
            resource.category = payload.category;
            resource.updated_at = get_timestamp();
            Ok(resource.clone())
        } else {
            Err(EczemaError::NotFound)
        }
    })
}

#[ic_cdk_macros::update]
fn delete_resource(id: u64) -> EczemaResult<()> {
    let caller = ic_cdk::caller();
    if !is_admin(caller) {
        return Err(EczemaError::Unauthorized);
    }

    ECZEMA_RESOURCES.with(|resources| {
        CATEGORY_INDEX.with(|category_index| {
            if let Some(resource) = resources.borrow_mut().remove(&id) {
                if let Some(category_ids) = category_index.borrow_mut().get_mut(&resource.category) {
                    category_ids.retain(|&x| x != id);
                }
                Ok(())
            } else {
                Err(EczemaError::NotFound)
            }
        })
    })
}

#[ic_cdk_macros::update]
fn verify_resource(id: u64) -> EczemaResult<EczemaResource> {
    let caller = ic_cdk::caller();
    if !is_admin(caller) {
        return Err(EczemaError::Unauthorized);
    }

    ECZEMA_RESOURCES.with(|resources| {
        let mut resources = resources.borrow_mut();
        if let Some(resource) = resources.get_mut(&id) {
            resource.verified = true;
            resource.updated_at = get_timestamp();
            Ok(resource.clone())
        } else {
            Err(EczemaError::NotFound)
        }
    })
}

#[ic_cdk_macros::query]
fn search_resources(query: String, page: usize) -> Vec<EczemaResource> {
    let query = query.to_lowercase();
    ECZEMA_RESOURCES.with(|resources| {
        resources
            .borrow()
            .values()
            .filter(|r| {
                r.title.to_lowercase().contains(&query) ||
                r.description.to_lowercase().contains(&query)
            })
            .skip(page * PAGE_SIZE)
            .take(PAGE_SIZE)
            .cloned()
            .collect()
    })
}

#[ic_cdk_macros::init]
fn init() {
    let caller = ic_cdk::caller();
    ADMINS.with(|admins| admins.borrow_mut().push(caller));
}

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {
    let resources = ECZEMA_RESOURCES.with(|r| r.borrow().clone());
    let category_index = CATEGORY_INDEX.with(|c| c.borrow().clone());
    let next_id = NEXT_ID.with(|n| *n.borrow());
    let admins = ADMINS.with(|a| a.borrow().clone());
    storage::stable_save((resources, category_index, next_id, admins)).unwrap();
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    let (resources, category_index, next_id, admins): (
        HashMap<u64, EczemaResource>,
        BTreeMap<ResourceCategory, Vec<u64>>,
        u64,
        Vec<Principal>,
    ) = storage::stable_restore().unwrap();
    ECZEMA_RESOURCES.with(|r| *r.borrow_mut() = resources);
    CATEGORY_INDEX.with(|c| *c.borrow_mut() = category_index);
    NEXT_ID.with(|n| *n.borrow_mut() = next_id);
    ADMINS.with(|a| *a.borrow_mut() = admins);
}

// Export the Candid interface
ic_cdk::export_candid!();