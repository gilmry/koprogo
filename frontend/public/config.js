// Runtime configuration for KoproGo Frontend
// This file can be modified after build to update API URL
window.__ENV__ = {
  API_URL: import.meta.env.PUBLIC_API_URL || "http://localhost:8080/api/v1"
};
