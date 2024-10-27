# Eczema Awareness Backend - ICP Blockchain

## Overview
This project provides a **Rust backend** for managing eczema awareness resources on the **Internet Computer (ICP)** blockchain. It supports operations such as creating, updating, verifying, and searching for eczema-related resources, including treatments, prevention tips, diet advice, and more. The backend uses **Candid** for data serialization and **IC-CDK** macros to interact with the Internet Computer.

---

## Features
- **Create** and manage eczema resources (e.g., treatment plans, research, testimonials).
- **Query** resources by ID, category, or search keywords.
- **Update** and verify resources.
- **List** all resources or filter them by category with **pagination** support.
- **Delete** outdated or unwanted resources.
- **Verify** resources for accuracy and trustworthiness.
- **Access Control** to restrict sensitive actions (e.g., verification, deletion) to admins.
- **Indexed data** for faster lookups by category.
- **Persistent storage** using `RefCell` for efficient resource management.
- **Upgrade Safety**: Handles canister upgrades smoothly with data persistence.
- **Optimized Search** with pagination for large datasets.

---

## Data Structures

- **EczemaResource**:  
  Represents a single resource with attributes like title, description, category, creation date, etc.

- **ResourceCategory**:  
  Enum representing the categories a resource can belong to, such as Treatment, Prevention, Research, DietAdvice, Testimonial, and MedicalAdvice.

- **EczemaError**:  
  Custom error types for resource operations, including specific types like `InvalidInput` and `Unauthorized` for better feedback.

---

## API Endpoints

| Endpoint                      | Type   | Description                          |
|-------------------------------|--------|--------------------------------------|
| `create_resource`             | Update | Add a new eczema resource with validation checks. |
| `get_resource(id: u64)`       | Query  | Retrieve a resource by its ID.      |
| `list_resources(page: usize)` | Query  | List resources with pagination.     |
| `list_resources_by_category`  | Query  | List resources by category with pagination. |
| `update_resource(id, payload)`| Update | Modify an existing resource with access control. |
| `delete_resource(id: u64)`    | Update | Remove a resource by ID with admin access required. |
| `verify_resource(id: u64)`    | Update | Mark a resource as verified (admin-only). |
| `search_resources(query, page)` | Query | Search resources by title/description with pagination. |

---

## Installation

1. Ensure you have the following installed:
   - Rust: [Install Rust](https://www.rust-lang.org/tools/install)
   - DFX SDK: [Install DFX](https://internetcomputer.org/docs/current/developer-docs/quickstart/dfx-install/)

2. Clone the repository:
   ```bash
   git clone https://github.com/PreciousMuemi/eczema-icp.git
   cd eczema-icp
   cd app
   ```

3. Install dependencies:
   ```bash
   cargo build
   ```

4. Deploy to ICP:
   ```bash
   dfx start --background
   dfx deploy
   ```

---

## Usage

After deployment, you can access the Candid UI via the link provided in the terminal. Alternatively, you can use the following methods to test the canister in the CLI:

1. **Create a resource**:

   ```bash
   dfx canister call eczema_awareness create_resource '(
   record {
      title = "Managing Eczema Flare-ups";
      description = "Tips and tricks for managing sudden eczema flare-ups";
      category = variant { Treatment };
   }
   )'
   ```

2. **Get a resource by ID**:
   ```bash
   dfx canister call eczema_awareness get_resource '(1)'
   ```

3. **List resources (paginated)**:
   ```bash
   dfx canister call eczema_awareness list_resources '(0)'
   ```

4. **List resources by category (paginated)**:
   ```bash
   dfx canister call eczema_awareness list_resources_by_category '(variant { DietAdvice }, 0)'
   ```

5. **Update a resource**:
   ```bash
      dfx canister call eczema_awareness update_resource '(1, 
   record {
      title = "Updated: Managing Eczema Flare-ups";
      description = "Updated tips for managing eczema flare-ups";
      category = variant { Treatment };
   }
   )'
   ```

6. **Search resources (paginated)**:
   ```bash
   dfx canister call eczema_awareness search_resources '("eczema", 0)'
   ```

7. **Verify a resource (admin only)**:
   ```bash
   dfx canister call eczema_awareness verify_resource '(1)'
   ```

8. **Delete a resource (admin only)**:
   ```bash
   dfx canister call eczema_awareness delete_resource '(1)'
   ```

## License
This project is open-source under the [MIT License](https://opensource.org/licenses/MIT).