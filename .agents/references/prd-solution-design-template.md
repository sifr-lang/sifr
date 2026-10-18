Use this template when creating a PRDS (a PRD and a solution design) for a feature or an epic (output is written into `/issues` folder):


## 📄 Product Requirements & Solution Design Template

---

### 🧭 1. Product Requirements

#### **Title**

*Enter project or feature name here*

---

#### **Objective / Problem Statement**

*What problem are you solving and why does it matter?*

---

#### **Constraints** *(business-wise)*

| Constraint                              | Rationale       |
| --------------------------------------- | --------------- |
| *e.g. data retention, compliance, SLAs* | *Justification* |

---

#### **Business Goals & Success Criteria (KPIs)**

| Metric                     | **Baseline (Today)** | **Target (Post-launch)** |
| -------------------------- | -------------------- | ------------------------ |
| *e.g. error rate, latency* | *% or duration*      | *% or duration*          |

---

#### **Scope**

##### ✅ Features In

1. *Feature A*
2. *Feature B*

##### ❌ Features Out

| Feature                          | Reason for Exclusion              |
| -------------------------------- | --------------------------------- |
| *Deferred or out-of-scope items* | *Reason (cost, time, dependency)* |

---

#### **Users / Stakeholders, Use-Cases & Dependencies**

| Persona     | Use-Case / Benefit | Dependencies      | **AC-ID** |
| ----------- | ------------------ | ----------------- | --------- |
| *User type* | *How they benefit* | *What’s required* | *AC-1*    |

---

### **Acceptance Criteria**

| **AC-ID** | Persona | Criterion *(Given / When / Then)*       |
| --------- | ------- | --------------------------------------- |
| *AC-1*    | *User*  | **Given** ... **When** ... **Then** ... |

---

## 🧠 2. Solution Design

### **2.1 Functional Requirements**

* *What the system must do*

---

### **2.2 Non-Functional Requirements**

| ID    | Requirement                                     |
| ----- | ----------------------------------------------- |
| NFR-1 | *Performance, scalability, observability, etc.* |

---

### **2.3 High-Level Architecture**

```
_Component A_
    ↓
_Component B_
    ↓
_Database / External System_
```

---

### **2.4 Detailed Component Design**

**📦 Component A**
*Responsibilities, interactions, and error handling.*

**⚙️ Component B**
*Responsibilities, interactions, and dependencies.*

**🗄️ Data Store**
*Schema, indexes, update strategy.*

**🚨 Alerting / Notifications**
*When alerts fire, thresholds, reset logic.*

---

### **2.5 Data Model**

```sql
CREATE TABLE table_name (
  id TEXT PRIMARY KEY,
  field_name TYPE NOT NULL
);
```

---

### **2.6 API Integration**

* *Endpoints, auth, pagination, rate limits, error formats.*

---

### **2.7 Error Handling & Monitoring**

* *Retries, logging, metrics, dashboards.*

---

### **2.8 Deployment Plan**

* *How it will be released, flagged, and monitored.*

---

### **2.9 Trade-offs & Alternatives**

| Option Considered | Pros   | Cons   | Rationale for Final Choice |
| ----------------- | ------ | ------ | -------------------------- |
| *Approach A*      | *Pros* | *Cons* | *Why chosen or not*        |

---

### **2.10 Testing Strategy** *(mapped to ACs)*

| **AC-ID** | Test Layer                 | Happy-Path Check    | Non-Happy / Edge Check | Tooling & Automation | Pass/Fail Gate |
| --------- | -------------------------- | ------------------- | ---------------------- | -------------------- | -------------- |
| *AC-1*    | *Unit / Integration / E2E* | *Expected behavior* | *Edge case or error*   | *Tool name*          | *Criteria*     |

**Additional NFR Tests**

* *Performance*
* *Chaos/resilience*
* *Security/compliance*
