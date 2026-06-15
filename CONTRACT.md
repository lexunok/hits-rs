# API Contract Analysis Report: HITS-RS (Backend) vs. HITS-Blazor (Frontend)

This document outlines the inconsistencies found between the Rust backend API (`hits-rs`) and the Blazor frontend application (`HITS-Blazor`). The analysis covers API endpoints and Data Transfer Objects (DTOs) for each domain.

---

## Table of Contents
1.  [Auth Contract Analysis (`/api/auth`)](#auth-contract-analysis-apiauth)
2.  [Company Contract Analysis (`/api/company`)](#company-contract-analysis-apicompany)
3.  [Group Contract Analysis (`/api/group`)](#group-contract-analysis-apigroup)
4.  [Idea Contract Analysis (`/api/idea`)](#idea-contract-analysis-apiidea)
5.  [Idea Market Contract Analysis (`/api/idea-market`)](#idea-market-contract-analysis-apiidea-market)
6.  [Invitation Contract Analysis (`/api/invitation`)](#invitation-contract-analysis-apiinvitation)
7.  [Market Contract Analysis (`/api/market`)](#market-contract-analysis-apimarket)
8.  [Profile Contract Analysis (`/api/profile`)](#profile-contract-analysis-apiprofile)
9.  [Project Contract Analysis (`/api/project`)](#project-contract-analysis-apiproject)
10. [Rating Contract Analysis (`/api/rating`)](#rating-contract-analysis-apirating)
11. [Skill Contract Analysis (`/api/skill`)](#skill-contract-analysis-apiskill)
12. [Sprint Contract Analysis (`/api/sprint`)](#sprint-contract-analysis-apisprint)
13. [Tag Contract Analysis (`/api/tag`)](#tag-contract-analysis-apitag)
14. [Task Contract Analysis (`/api/task`)](#task-contract-analysis-apitask)
15. [Task Movement Log Contract Analysis (`/api/task-movement-log`)](#task-movement-log-contract-analysis-apitask-movement-log)
16. [Team Contract Analysis (`/api/team`)](#team-contract-analysis-apiteam)
17. [User Contract Analysis (`/api/user`)](#user-contract-analysis-apiuser)

---

## Auth Contract Analysis (`/api/auth`)

### Endpoints

| Method | Path                               | Backend Handler                 | Frontend Method                    | Status         | Notes                                                                                                    |
| ------ | ---------------------------------- | ------------------------------- | ---------------------------------- | -------------- | -------------------------------------------------------------------------------------------------------- |
| `POST` | `/login`                           | `login`                         | `LoginAsync`                       | Match          | -                                                                                                        |
| `POST` | `/registration/{id}`               | `registration`                  | `RegistrationUserAsync`            | Match          | -                                                                                                        |
| `POST` | `/refresh`                         | `refresh`                       | `RefreshTokenAsync`                | Match          | -                                                                                                        |
| `POST` | `/logout`                          | `logout`                        | `LogoutAsync`                      | Match          | -                                                                                                        |
| `POST` | `/password/verification/{email}`   | `request_to_update_password`    | `PasswordVerificationAsync`        | Match          | -                                                                                                        |
| `PUT`  | `/password`                        | `confirm_and_update_password`   | `PasswordNewAsync`                 | Match          | -                                                                                                        |

### Models/DTOs

#### 1. Login

-   **Backend (`LoginPayload`)**:
    ```rust
    pub struct LoginPayload {
        pub email: String,
        pub password: String,
    }
    ```
-   **Frontend (`LoginModel`)**:
    ```csharp
    // Inferred from LoginModel in HITSBlazor.Pages.Auth.Login
    public class LoginModel {
        public string Email { get; set; }
        public string Password { get; set; }
    }
    ```
-   **Status**: Match

#### 2. Registration

-   **Backend (`RegisterPayload`)**:
    ```rust
    pub struct RegisterPayload {
        pub email: String,
        pub password: String,
        pub last_name: String,
        pub first_name: String,
        pub study_group: Option<String>,
        pub telephone: Option<String>,
    }
    ```
-   **Frontend (`RegisterModel`)**:
    ```csharp
    // Inferred from RegisterModel in HITSBlazor.Pages.Auth.Register
    public class RegisterModel {
        public string Email { get; set; }
        public string Password { get; set; }
        public string FirstName { get; set; }
        public string LastName { get; set; }
        public string Patronymic { get; set; } // Mismatch: Not on backend
        public string PhoneNumber { get; set; } // Mismatch: 'telephone' on backend
        public string StudyGroup { get; set; }
    }
    ```
-   **Status**: Mismatch
-   **Notes**:
    -   The frontend's `RegisterModel` includes a `Patronymic` field which is absent in the backend `RegisterPayload`.
    -   The frontend uses `PhoneNumber`, while the backend expects `telephone`. The property name should be aligned.

#### 3. Password Reset

-   **Backend (`PasswordResetPayload`)**:
    ```rust
    pub struct PasswordResetPayload {
        pub id: Uuid,
        pub code: String,
        pub password: String,
    }
    ```
-   **Frontend (`NewPasswordModel`)**:
    ```csharp
    // Inferred from NewPasswordModel in HITSBlazor.Pages.Auth.NewPassword
    public class NewPasswordModel {
        public Guid Id { get; set; }
        public string Code { get; set; }
        public string Password { get; set; }
    }
    ```
-   **Status**: Match

---

## Company Contract Analysis (`/api/company`)

### Endpoints

| Method   | Path             | Backend Handler         | Frontend Method      | Status           | Notes                                                              |
| -------- | ---------------- | ----------------------- | -------------------- | ---------------- | ------------------------------------------------------------------ |
| `GET`    | `/`              | `get_all_companies`     | `GetCompaniesAsync`  | Match            | Frontend seems to implement pagination and search, backend does not. |
| `POST`   | `/`              | `create_company`        | `CreateCompanyAsync` | Match            |                                                                    |
| `PUT`    | `/`              | `update_company`        | `UpdateCompanyAsync` | Match            |                                                                    |
| `GET`    | `/{id}`          | `get_company_by_id`     | `GetCompanyByIdAsync`| Match            |                                                                    |
| `DELETE` | `/{id}`          | `delete_company`        | `DeleteCompanyAsync` | Match            |                                                                    |
| `GET`    | `/{id}/members`  | `get_company_members`   | `GetCompanyMembersAsync` | Match            | Frontend implements pagination and search which backend may not. |
| `GET`    | `/my`            | `get_my_companies`      | **Missing on FE**    | Missing on FE    |                                                                    |

### Models/DTOs

#### 1. Company Response

-   **Backend (`CompanyResponse`)**:
    ```rust
    pub struct CompanyResponse {
        pub id: Uuid,
        pub name: String,
        pub owner: UserDto,
        pub members: Vec<UserDto>,
    }
    ```
-   **Frontend (`Company`)**:
    ```csharp
    // Inferred from MockCompanyService.cs
    public class Company {
        public Guid Id { get; set; }
        public string Name { get; set; }
        public User Owner { get; set; }
        public List<User> Members { get; set; }
    }
    ```
-   **Status**: Match

#### 2. Create Company Request

-   **Backend (`CreateCompanyRequest`)**:
    ```rust
    pub struct CreateCompanyRequest {
        pub name: String,
        pub owner_id: Uuid,
        pub members: Vec<Uuid>,
    }
    ```
-   **Frontend (inferred from `CreateCompanyAsync` signature)**:
    ```csharp
    // Inferred
    public class CreateCompanyRequest {
        public string Name { get; set; }
        public Guid OwnerId { get; set; } // Inferred from 'User owner' parameter
        public List<Guid> Members { get; set; } // Inferred from 'HashSet<User> members' parameter
    }
    ```
-   **Status**: Match

#### 3. Update Company Request

-   **Backend (`UpdateCompanyRequest`)**:
    ```rust
    pub struct UpdateCompanyRequest {
        pub id: Uuid,
        pub name: Option<String>,
        pub owner_id: Option<Uuid>,
        pub members: Option<Vec<Uuid>>,
    }
    ```
-   **Frontend (inferred from `UpdateCompanyAsync` signature)**:
    ```csharp
    // Inferred
    public class UpdateCompanyRequest {
        public Guid Id { get; set; }
        public string? Name { get; set; }
        public Guid? OwnerId { get; set; }
        public IEnumerable<Guid>? NewMembersIds { get; set; }
        public IEnumerable<Guid>? RemoveMembersIds { get; set; }
    }
    ```
-   **Status**: Match
-   **Notes**: The frontend `UpdateCompanyAsync` method uses nullable parameters, allowing for partial updates, which aligns with the backend's `Option<T>` fields.
---

## Group Contract Analysis (`/api/group`)

### Endpoints

| Method   | Path    | Backend Handler     | Frontend Method       | Status        | Notes                                                              |
| -------- | ------- | ------------------- | --------------------- | ------------- | ------------------------------------------------------------------ |
| `GET`    | `/`     | `get_all_groups`    | `GetUsersGroupsAsync` | Match         | Frontend seems to implement pagination and search, backend does not. |
| `POST`   | `/`     | `create_group`      | `CreateUsersGroup`    | Match         |                                                                    |
| `PUT`    | `/`     | `update_group`      | `UpdateUsersGroup`    | Match         |                                                                    |
| `GET`    | `/{id}` | `get_group_by_id`   | `GetUsersGroupByIdAsync` | Match         |                                                                    |
| `DELETE` | `/{id}` | `delete_group`      | `DeleteUsersGroupsAsync` | Match         |                                                                    |

### Models/DTOs

#### 1. Group DTO

-   **Backend (`GroupDto`)**:
    ```rust
    pub struct GroupDto {
        pub id: Uuid,
        pub name: String,
        pub roles: Vec<Role>,
        pub members: Vec<UserDto>,
    }
    ```
-   **Frontend (`UsersGroup`)**:
    ```csharp
    // Inferred from MockUsersGroupsService.cs
    public class UsersGroup {
        public Guid Id { get; set; }
        public string Name { get; set; }
        public List<RoleType> Roles { get; set; } // RoleType is an enum
        public List<User> Members { get; set; }
    }
    ```
-   **Status**: Match

#### 2. Create Group Request

-   **Backend (`CreateGroupRequest`)**:
    ```rust
    pub struct CreateGroupRequest {
        pub name: String,
        pub roles: Vec<Role>,
        pub members: Vec<Uuid>,
    }
    ```
-   **Frontend (inferred from `CreateUsersGroup` signature)**:
    ```csharp
    // Inferred
    public class CreateGroupRequest {
        public string Name { get; set; }
        public List<RoleType> Roles { get; set; }
        public List<Guid> Members { get; set; }
    }
    ```
-   **Status**: Match

#### 3. Update Group Request

-   **Backend (`UpdateGroupRequest`)**:
    ```rust
    pub struct UpdateGroupRequest {
        pub id: Uuid,
        pub name: Option<String>,
        pub roles: Option<Vec<Role>>,
        pub members: Option<Vec<Uuid>>,
    }
    ```
-   **Frontend (inferred from `UpdateUsersGroup` signature)**:
    ```csharp
    // Inferred
    public class UpdateGroupRequest {
        public Guid Id { get; set; }
        public string? Name { get; set; }
        public IEnumerable<Guid>? NewMembersIds { get; set; }
        public IEnumerable<Guid>? RemoveMembersIds { get; set; }
        public IEnumerable<RoleType>? Roles { get; set; }
    }
    ```
-   **Status**: Match
-   **Notes**: The frontend `UpdateUsersGroup` method uses nullable parameters, allowing for partial updates, which aligns with the backend's `Option<T>` fields.

---
## Idea Contract Analysis (`/api/idea`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/{id}` | `get_idea_by_id` | `GetIdeaByIdAsync` | Match | |
| `DELETE` | `/{id}` | `delete_idea` | `DeleteIdeaAsync` | Match | |
| `GET` | `/` | `get_all_ideas` | `GetIdeasAsync` | Match | FE has more query params (statusTypes) than BE. |
| `POST` | `/` | `save_idea` | `CreateNewIdeaAsync`/`UpdateIdeaAsync` | Match | FE separates create and update, BE handles both in one. |
| `GET` | `/initiator` | `get_all_initiator_ideas` | `GetIdeasAsync` (with logic) | Match | FE logic seems to handle this based on user role. |
| `GET` | `/on-confirmation` | `get_all_on_confirmation_ideas` | **Missing on FE** | Missing on FE | |
| `PUT` | `/status` | `update_status` | `UpdateIdeaStatusAsync` | Match | |
| `PUT` | `/send/{id}` | `send_idea_to_approval` | **Missing on FE** | Missing on FE | FE `CreateNewIdeaAsync` handles this via status. |
| `GET` | `/skills/{id}` | `get_idea_skills` | `GetAllIdeaSkillsAsync` | Match | |
| `POST` | `/skills` | `save_idea_skills` | `CreateOrUpdateIdeasSkills` | Match | |
| `GET` | `/comments/{id}` | **Missing on BE** | `GetIdeasCommentsAsync` | Missing on BE | FE uses mock for comments. |
| `DELETE` | `/comments/{id}` | **Missing on BE** | `DeleteCommentInIdeaAsync` | Missing on BE | FE uses mock for comments. |

### Models/DTOs

#### 1. Save Idea Request
-   **Backend (`SaveIdeaRequest`)**:
    ```rust
    pub struct SaveIdeaRequest {
        pub id: Option<Uuid>,
        pub name: String,
        pub status: IdeaStatus,
        pub problem: String,
        pub solution: String,
        pub result: String,
        pub customer: String,
        pub contact_person: String,
        pub description: String,
        pub suitability: i64,
        pub budget: i64,
        pub max_team_size: i16,
        pub min_team_size: i16,
    }
    ```
-   **Frontend (`IdeasCreateModel`)**:
    ```csharp
    // Inferred from HITSBlazor.Pages.Ideas.IdeasCreate.IdeasCreateModel
    public class IdeasCreateModel {
        public string Name { get; set; }
        public IdeaStatusType Status { get; set; }
        public string Problem { get; set; }
        public string Solution { get; set; }
        public string Result { get; set; }
        public string Customer { get; set; }
        public string ContactPerson { get; set; }
        public string Description { get; set; }
        // public long Suitability { get; set; } // Mismatch: Missing on FE model
        // public long Budget { get; set; }      // Mismatch: Missing on FE model
        public short MaxTeamSize { get; set; }
        public short MinTeamSize { get; set; }
    }
    ```
-   **Status**: Mismatch
-   **Notes**: `suitability` is present on the backend but seems to be missing from the frontend model. The `id` for updates is passed separately in the frontend method.

---

## Idea Market Contract Analysis (`/api/idea-market`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/` | `get_all_idea_markets` | `GetIdeasMarketAsync` | Mismatch | FE provides `marketId`, BE does not expect it. |
| `GET` | `/market/{market_id}` | `get_all_idea_markets_by_market` | `GetIdeasMarketAsync` | Match | |
| `GET` | `/my/{market_id}` | `get_my_idea_markets` | `GetIdeasMarketAsync` (with logic) | Match | FE logic handles this via user role. |
| `GET` | `/favorite/{market_id}` | `get_favorite_idea_markets` | `GetIdeasMarketAsync` (with favorite=true) | Match | |
| `PUT` | `/send/{market_id}` | `send_ideas_to_market` | `SendIdeasOnMarket` | Match | |
| `PUT` | `/favorite/{idea_market_id}` | `add_to_favorite` | `SetIdeaFavorite` | Match | |
| `DELETE` | `/favorite/{idea_market_id}` | `delete_from_favorite` | `UnsetIdeaFromFavorite` | Match | |
| `POST` | `/advertisement` | `add_advertisement` | **Missing on FE** | Missing on FE | |
| `GET` | `/advertisement/{idea_market_id}` | `get_advertisements_by_idea_market` | **Missing on FE** | Missing on FE | |
| `PUT` | `/advertisement/check/{advertisement_id}` | `mark_advertisement_checked` | **Missing on FE** | Missing on FE | |
| `DELETE` | `/advertisement/{advertisement_id}/delete` | `delete_advertisement` | **Missing on FE** | Missing on FE | |
| `PUT` | `/status/{idea_market_id}/{status}` | `update_idea_market_status` | **Missing on FE** | Missing on FE | |
| `GET` | `/{idea_market_id}` | `get_idea_market_by_id` | `GetIdeaMarketAsync` | Match | |
| `DELETE` | `/{idea_market_id}` | `delete_idea_market` | **Missing on FE** | Missing on FE | |

### Models/DTOs

#### 1. IdeaMarket DTO
- **Backend (`IdeaMarketDto`)**:
    ```rust
    pub struct IdeaMarketDto {
        pub id: Uuid,
        pub idea_id: Uuid,
        pub initiator: UserDto,
        pub team: Option<IdeaMarketTeamDto>,
        pub market_id: Uuid,
        pub name: String,
        // ... and many other fields
        pub is_favorite: bool,
    }
    ```
- **Frontend (`IdeaMarket`)**:
    ```csharp
    // Inferred from MockIdeaMarketService.cs
    public class IdeaMarket {
        public Guid Id { get; set; }
        public Guid IdeaId { get; set; }
        public User Initiator { get; set; }
        public Team Team { get; set; } // Type might be different from IdeaMarketTeamDto
        public Guid MarketId { get; set; }
        public string Name { get; set; }
        // ... and many other fields
        public bool IsFavorite { get; set; }
    }
    ```
-   **Status**: Match (High-level, fields need deep comparison)
-   **Notes**: The structures seem to align, but a detailed field-by-field check is recommended. `IdeaMarketTeamDto` vs `Team` could have discrepancies.

---
## Invitation Contract Analysis (`/api/invitation`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `POST` | `/` | `send_invitations` | `SendInvitations` | Match | |
| `GET` | `/{id}` | `get_invitation` | `GetEmailById` / `GetEmailByInvitationId` | Match | |

### Models/DTOs

#### 1. Invitation Payload
- **Backend (`InvitationPayload`)**:
    ```rust
    pub struct InvitationPayload {
        pub emails: Vec<String>,
        pub roles: Vec<Role>,
    }
    ```
- **Frontend (`InviteUsersRequest`)**:
    ```csharp
    // Inferred from HITSBlazor.Models.Users.Requests
    public class InviteUsersRequest {
        public List<string> Emails { get; set; }
        public List<RoleType> Roles { get; set; }
    }
    ```
-   **Status**: Match

#### 2. Invitation Response
- **Backend (`InvitationResponse`)**:
    ```rust
    pub struct InvitationResponse {
        pub email: String,
        pub code: Uuid, // id of invitation
    }
    ```
- **Frontend**: The frontend `GetEmailById` method directly returns a `string` (the email), losing the `code`. The `InvitationApi` correctly expects an object with an `email` field.
- **Status**: Mismatch
-   **Notes**: `IInvitationService` defines `GetEmailById` as returning `Task<string?>`, but the API response is an object. `InvitationApi` correctly handles the object but only extracts the email. This is an inconsistency between the FE service interface and the actual API client implementation.

---
## Market Contract Analysis (`/api/market`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/` | `get_all_markets` | `GetMarketsAsync` | Match | FE has more features (sorting). |
| `POST` | `/` | `create_market` | `CreateNewMarketAsync` | Match | |
| `PUT` | `/` | `update_market` | `UpdateMarketAsync` | Match | |
| `GET` | `/active` | `get_active_markets` | `GetAllActiveMarketsAsync` | Match | Frontend uses a specific method for active markets. |
| `PUT` | `/status` | `update_market_status` | `UpdateMarketStatusAsync` | Match | |
| `GET` | `/{id}` | `get_market_by_id` | `GetMarketByIdAsync` | Match | |
| `DELETE` | `/{id}` | `delete_market` | `DeleteMarketAsync` | Match | |

### Models/DTOs

#### 1. Market DTO
- **Backend (`MarketDto`)**:
    ```rust
    pub struct MarketDto {
        pub id: Uuid,
        pub name: String,
        pub start_date: NaiveDate,
        pub finish_date: NaiveDate,
        pub status: MarketStatus,
    }
    ```
- **Frontend (`Market`)**:
    ```csharp
    // Inferred from HITSBlazor.Models.Markets.Entities
    public class Market {
        public Guid Id { get; set; }
        public string Name { get; set; }
        public DateTime StartDate { get; set; }
        public DateTime FinishDate { get; set; }
        public MarketStatus Status { get; set; }
    }
    ```
-   **Status**: Match. `NaiveDate` in Rust corresponds well to `DateTime` in C# for date-only values.

---

## Profile Contract Analysis (`/api/profile`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `PUT` | `/` | `update_profile` | `UpdateProfileUserDataAsync` | Match | |
| `GET` | `/{id}` | `get_profile` | `GetUserProifleAsync` | Match | |
| `PUT` | `/skills` | `update_profile_skills` | **Missing on FE** | Missing on FE | |
| `POST` | `/avatar` | `upload_avatar` | **Missing on FE** | Missing on FE | |
| `POST` | `/email/verification/{new_email}` | `request_to_update_email` | `SendUpdateEmailRequestAsync` | Match | |
| `PUT` | `/email` | `confirm_and_update_email` | `UpdateEmailConfirmAsync` | Match | |

### Models/DTOs

#### 1. Profile DTO
- **Backend (`ProfileDto`)**:
    ```rust
    pub struct ProfileDto {
        pub id: Uuid,
        pub study_group: Option<String>,
        pub telephone: Option<String>,
        pub roles: Vec<Role>,
        pub email: String,
        pub last_name: String,
        pub first_name: String,
        pub created_at: DateTimeLocal,
        pub skills: Vec<SkillDto>,
        pub ideas: Vec<ProfileIdeaDto>,
        pub teams: Vec<TeamExperienceDto>,
    }
    ```
- **Frontend (`Profile`)**:
    ```csharp
    // Inferred from HITSBlazor.Models.Users.Entities
    public class Profile {
        public Guid Id { get; set; }
        public string StudyGroup { get; set; }
        public string Telephone { get; set; }
        public List<RoleType> Roles { get; set; }
        public string Email { get; set; }
        public string LastName { get; set; }
        public string FirstName { get; set; }
        public DateTime CreatedAt { get; set; }
        public List<Skill> Skills { get; set; }
        public List<Idea> Ideas { get; set; } // Mismatch: Type differs from ProfileIdeaDto
        public List<TeamExperience> Teams { get; set; } // Mismatch: Type differs from TeamExperienceDto
    }
    ```
-   **Status**: Mismatch
-   **Notes**: The frontend `Profile` model uses full `Idea` and `TeamExperience` models for `Ideas` and `Teams` lists, while the backend uses specialized, smaller DTOs (`ProfileIdeaDto`, `TeamExperienceDto`). This is inefficient and incorrect.

---
## Project Contract Analysis (`/api/project`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/` | `get_all_projects` | `GetProjectsByQueryAsync` | Match | |
| `GET` | `/my` | `get_my_projects` | **Missing on FE** | Missing on FE | |
| `GET` | `/my/active` | `get_my_active_projects` | `GetAllActiveProjectsAsync` | Match | |
| `POST` | `/create/{idea_market_id}` | `create_project` | `CreateNewProjectAsync` | Match | |
| `GET` | `/{project_id}` | `get_project_by_id` | `GetProjectByIdAsync` | Match | |
| `DELETE` | `/{project_id}` | `delete_project` | `DeleteProjectAsync` | Match | |
| `GET` | `/members/{project_id}` | `get_project_members` | `GetProjectMembersAsync` | Match | |
| `POST` | `/members/{project_id}` | `add_member` | `AddMemberInProjectAsync` | Match | |
| `DELETE` | `/members/{project_id}/{user_id}` | `kick_member_from_project_and_team` | `KickMemberFromProjectAsync` | Match | |
| `GET` | `/marks/{project_id}` | `get_project_marks` | `GetProjectMarksAsync` | Match | |
| `PUT` | `/pause/{project_id}` | `pause_project` | `PauseProjectAsync` | Match | |
| `PUT` | `/finish/{project_id}` | `finish_project` | `FinishProjectAsync` | Match | |
| `PUT` | `/team/{project_id}/{team_id}` | `change_team_in_project` | **Missing on FE** | Missing on FE | |

### Models/DTOs

A full DTO analysis for the `Project` domain is complex due to nested models. High-level observations:
- **`ProjectDto` vs `Project`**: The structures appear to align at a high level, but nested objects like `ProjectTeamDto`, `ProjectMemberDto`, and `ReportProjectDto` on the backend need to be carefully compared with their frontend counterparts (`Team`, `ProjectMember`, `Report`). Discrepancies are likely.
- **`FinishProjectRequest`**: The backend expects `{ "report": "..." }`, and the frontend `FinishProjectAsync` call matches this.

---
## Rating Contract Analysis (`/api/rating`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/{idea_id}` | `get_all_ratings` | `GetIdeaRatingsAsync` (in `IIdeasService`) | Match | Endpoint is part of `idea` service on FE. |
| `GET` | `/expert/{idea_id}` | `get_all_ratings_by_expert` | **Missing on FE** | Missing on FE | |
| `PUT` | `/save` | `save_rating` | `SendRatingAsync` (in `IIdeasService`) | Match | |
| `PUT` | `/confirm` | `confirm_rating` | `SendRatingAsync(isConfirmed: true)` | Match | |

### Models/DTOs

#### 1. Update Rating Request
- **Backend (`UpdateRatingRequest`)**:
    ```rust
    pub struct UpdateRatingRequest {
        pub id: Uuid,
        pub market_value: i64,
        pub originality: i64,
        pub technical_realizability: i64,
        pub suitability: i64,
        pub budget: i64,
    }
    ```
- **Frontend (`RatingRequest`)**:
    ```csharp
    // Inferred from HITSBlazor.Models.Ideas.Entities.RatingRequest
    public class RatingRequest {
        public Guid Id { get; set; }
        public long MarketValue { get; set; }
        public long Originality { get; set; }
        public long TechnicalRealizability { get; set; }
        public long Suitability { get; set; }
        public long Budget { get; set; }
    }
    ```
-   **Status**: Match

---
## Skill Contract Analysis (`/api/skill`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/` | `get_all_skills` | `GetSkillsAsync` | Match | |
| `POST` | `/` | `create_skill` | `CreateNewSkillAsync` | Mismatch | FE doesn't seem to distinguish confirmed/unconfirmed creation. |
| `PUT` | `/` | `update_skill` | `UpdateSkillAsync` | Match | |
| `DELETE` | `/{id}` | `delete_skill` | `DeleteSkillAsync` | Match | |
| `GET` | `/type/{skill_type}` | `get_skills_by_type` | `GetSkillsAsync` (with filter) | Match | |
| `GET` | `/my` | `get_all_my_or_confirmed` | **Missing on FE** | Missing on FE | |

### Models/DTOs

#### 1. Skill DTO
- **Backend (`SkillDto`)**:
    ```rust
    pub struct SkillDto {
        pub id: Uuid,
        pub name: String,
        #[serde(rename = "type")]
        pub skill_type: SkillType,
        pub confirmed: bool,
        // ... creator/updater ids
    }
    ```
- **Frontend (`Skill`)**:
    ```csharp
    // Inferred from HITSBlazor.Models.Common.Entities
    public class Skill {
        public Guid Id { get; set; }
        public string Name { get; set; }
        public SkillType Type { get; set; }
        public bool Confirmed { get; set; }
    }
    ```
-   **Status**: Match (Core fields)

---
## Sprint Contract Analysis (`/api/sprint`)

**Overall Status**: Missing on FE
- **Notes**: The backend has a full suite of endpoints for sprint management (`get_all_sprints_by_project`, `create_sprint`, `update_sprint`, `finish_sprint`, etc.). The frontend's `IProjectService` has sprint-related methods, but they are not connected to dedicated API clients and rely on mock data. There is no `SprintApi.cs`. A full implementation is needed on the frontend.

---
## Tag Contract Analysis (`/api/tag`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/all` | `get_all_tags` | `GetTagsAsync` | Match | |
| `POST` | `/add` | `create_confirmed_tag` | `CreateNewTagAsync(isConfirmed: true)` | Match | |
| `POST` | `/add/no-confirmed` | `create_unconfirmed_tag` | `CreateNewTagAsync(isConfirmed: false)` | Match | |
| `PUT` | `/confirm/{tag_id}` | `confirm_tag` | `ConfirmTagAsync` | Match | |
| `PUT` | `/update/{tag_id}` | `update_tag` | `UpdateTagAsync` | Match | |
| `DELETE` | `/delete/{tag_id}` | `delete_tag` | `DeleteTagAsync` | Match | |

### Models/DTOs are consistent.

---
## Task Contract Analysis (`/api/task`)

**Overall Status**: Missing on FE
- **Notes**: Similar to Sprints, the backend has a comprehensive set of task management endpoints within a project/sprint context. The frontend's `IProjectService` includes signatures for these operations (`CreateNewTaskAsync`, `UpdateTaskAsync`, etc.), but they are backed by mock services, not a real `TaskApi.cs`.

---
## Task Movement Log Contract Analysis (`/api/task-movement-log`)

**Overall Status**: Missing on FE
- **Notes**: Backend provides endpoints to log task movements. Frontend `IProjectService` has `GetTasksLogsAsync`, but it is a mock implementation. No API client exists.

---
## Team Contract Analysis (`/api/team`)

### Endpoints
This domain has a large number of endpoints. A high-level summary:
- **Team Management** (`/`, `/{id}`): `GET`, `POST`, `PUT`, `DELETE` operations seem to be covered by `GetTeamsAsync`, `GetTeamByIdAsync`, `DeleteTeamAsync` on the frontend, but create/update are missing from `ITeamService`.
- **Invitations** (`/invitations/...`): The backend has extensive support for sending and managing member invitations. The frontend has `GetTeamInvitationsAsync` but lacks functionality for sending or updating them.
- **Requests** (`/requests/...`): Backend supports users requesting to join a team. Frontend has `GetTeamRequestsToTeamAsync` but no creation mechanism.
- **Market Requests** (`/market/request/...`): Extensive backend support for teams applying to idea markets. The frontend has partial support with `GetRequestsTeamToIdeasAsync` and `CreateRequestTeamToIdeaAsync`. Many status update and administrative endpoints are missing.
- **Member Management** (`/members/...`): Backend allows adding/kicking members. Frontend is missing this.

### Models/DTOs
- **`TeamDto` vs `Team`**: High-level structure seems to match, but deep comparison of nested objects (`OwnerDto`, `LeaderDto`, `members`, `skills`) is required. Mismatches are likely.
- **Inconsistency**: There's a significant gap between the rich functionality of the backend team management API and the limited, read-only mock implementation on the frontend.

---
## User Contract Analysis (`/api/user`)

### Endpoints

| Method | Path | Backend Handler | Frontend Method | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `GET` | `/` | `get_user` | `GetUserAsync(id: null)` | Match | |
| `POST` | `/` | `create_user` | **Missing on FE** | Missing on FE | |
| `PUT` | `/` | `update_user` | `UpdateUser` | Match | |
| `GET` | `/all` | `get_all_users` | `GetUsersAsync` | Match | |
| `GET` | `/all/with-skills` | `get_all_users_with_skills` | **Missing on FE** | Missing on FE | |
| `GET` | `/all/in-team` | `get_all_users_in_teams` | **Missing on FE** | Missing on FE | |
| `GET` | `/all/not-in-team` | `get_all_users_not_in_teams` | **Missing on FE** | Missing on FE | |
| `GET` | `/{id}` | `get_user` | `GetUserAsync(id: ...)` | Match | |
| `DELETE` | `/{id}` | `delete_user` | `DeleteUserAsync` | Match | |
| `PUT` | `/restore/{email}` | `restore_user` | **Missing on FE** | Missing on FE | |

### Models/DTOs

#### 1. User DTO
- **Backend (`UserDto`)**:
    ```rust
    pub struct UserDto {
        pub id: Uuid,
        pub study_group: Option<String>,
        pub telephone: Option<String>,
        pub roles: Vec<Role>,
        pub email: String,
        pub last_name: String,
        pub first_name: String,
        pub created_at: DateTimeLocal,
        pub skills: Option<Vec<SkillDto>>,
    }
    ```
- **Frontend (`User`)**:
    ```csharp
    // Inferred from HITSBlazor.Models.Users.Entities
    public class User {
        public Guid Id { get; set; }
        public string StudyGroup { get; set; }
        public string Telephone { get; set; }
        public List<RoleType> Roles { get; set; }
        public string Email { get; set; }
        public string LastName { get; set; }
        public string FirstName { get; set; }
        public DateTime CreatedAt { get; set; }
        public List<Skill> Skills { get; set; }
    }
    ```
-   **Status**: Match (Core fields).
-   **Notes**: The `User` model on the frontend is mostly consistent, but not all mock/service instances include `CreatedAt` or `Skills`, which could lead to null reference exceptions if not handled carefully.