# Oridion Protocol

**Oridion** is a privacy-focused protocol on Solana that obfuscates deposit paths using randomized planetary hops.  
It combines an **Anchor smart contract** with a backend automation layer powered by **AWS**.

---

## 🔐 Privacy Design Philosophy

### Purpose
Oridion provides **transaction obfuscation** on Solana — making it difficult for casual observers to trace fund origins — while ensuring transactions remain **auditable** when necessary.

This is **privacy, not anonymity**: Oridion shields everyday flows without undermining compliance.

---

### Design Goals

- Break transaction patterns to reduce backward traceability
- Avoid classification as a “mixer” or prohibited laundering tool
- Keep usage **cheap and simple** — regardless of deposit size
- **Submit once and forget** — no passcodes or special keys required for withdrawals

---

## 🧱 System Architecture

### 1. 🤖 Automated Pod Delivery
Users specify only the **amount**, **destination**, and **delay** at launch.

**How it works:**
- The backend system automatically executes randomized hops during the delay period
- Once the delay expires, the system finalizes delivery by auto-landing the pod
- Users do not need to manually intervene or track intermediate states

**Benefits:**
- “Submit once, forget” user experience
- Consistent delivery regardless of deposit size
- Full automation increases obfuscation while reducing friction for non-technical users

---

### 2. 🎟️ Hash Tickets for Withdrawals
Each withdrawal or landing requires presenting a **hash-based ticket** that is generated once the pod is ready to be withdrawn.

**Purpose:**
- Ensures there is **no direct link** between the original deposit creator and the withdrawal transaction
- Breaks simple address-based tracing attempts
- Strengthens privacy guarantees without compromising auditability

**Design Notes:**
- The ticket is derived as a short hash commitment (e.g., pod ID + destination + lamports)
- Stored in a lightweight PDA ring buffer to prevent replay attacks
- Checked on withdrawal; once used, the ticket is deleted/invalidated

---

### 3. 🪐 System Orbits
Oridion injects internal “orbit” funds that constantly hop between planets and stars.

**Purpose:**
- Introduce **noise and traffic**
- Blend real user deposits with system movement to mask patterns

---

### 4. 🧭 Minimal On-Chain Logging
Core logic executes with **minimal event emission or logs**.

**Effect:**
- Makes activity harder to follow
- Requires manual state inspection, raising the bar for tracing attempts

---

## ⚖️ Legal Alignment

**Oridion is _not_ a mixer.** It is a transaction obfuscation protocol built with regulatory safety in mind.

### It is designed to:
- Add privacy layers for day-to-day users
- Be simple and easy to use for non-technical users
- Require no passcodes or special keys to withdraw funds
- Avoid intent to permanently hide or anonymize origins

### ✅ This ensures:
- Compatibility with regulated platforms
- Alignment with modern Web3 privacy expectations
- Reduced risk of regulatory classification as a prohibited tool

---

## 📬 Contact

Have questions, integration needs, or want to contribute?  
Reach out or open an issue.