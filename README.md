# XPCTL

![Rust](https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/mikeshootzz/xpctl/workflow.yaml)

**XPCTL** is a terminal interface for XPipe, enabling streamlined management of connections and remote operations.

---

## 🚀 Installation

### For macOS/Linux:

```bash
curl -L https://raw.githubusercontent.com/mikeshootzz/xpctl/main/install-scripts/unix.sh | bash
```

### For Windows:

```powershell
Invoke-Expression (Invoke-WebRequest -Uri https://raw.githubusercontent.com/mikeshootzz/xpctl/main/install-scripts/windows.ps1 -UseBasicParsing).Content
```

---

## ⚙️ Environment Variables

Before using XPCTL, set the following environment variable:

```bash
export XPIPE_API_KEY="your-api-key-here"
```

---

## 🗺️ Roadmap

- [x] **Basic SSH connections**
- [ ] **Docker Container support**
- [ ] **Kubernetes support**
- [ ] **Add and remove hosts**

---

## 📄 License

XPCTL is licensed under the [Apache 2.0 License](https://opensource.org/licenses/Apache-2.0).

---
