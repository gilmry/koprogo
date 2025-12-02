=========================================================================
Issue #43: feat(infra): Advanced security hardening (fail2ban, WAF, IDS)
=========================================================================

:State: **OPEN**
:Milestone: Jalon 1: Sécurité & GDPR 🔒
:Labels: phase:vps,track:infrastructure priority:high
:Assignees: Unassigned
:Created: 2025-10-27
:Updated: 2025-11-17
:URL: `View on GitHub <https://github.com/gilmry/koprogo/issues/43>`_

Description
===========

.. raw:: html

   <div class="github-issue-body">

::

   ## Context
   
   Current security implementation (60% complete):
   - ✅ UFW firewall (ports 22, 80, 443)
   - ✅ fail2ban installed (default config)
   - ✅ Non-root Docker containers
   - ✅ Traefik security headers (HSTS, X-Frame-Options, etc.)
   - ✅ Rate limiting (100 req/min per IP)
   
   **Missing critical security layers:**
   - ❌ Custom fail2ban jails for application-specific attacks
   - ❌ Web Application Firewall (WAF) rules
   - ❌ Intrusion Detection System (IDS)
   - ❌ SSH hardening (key-only, 2FA optional)
   - ❌ Security audit automation
   
   ## Objective
   
   Harden VPS security for production deployment with defense-in-depth strategy.
   
   ## Implementation Plan
   
   ### 1. fail2ban Custom Jails
   
   **Current:** fail2ban installed with default jails (SSH only)
   
   **Add custom jails:**
   
   **File:** `infrastructure/ansible/templates/fail2ban-koprogo.conf.j2`
   
   ```ini
   # /etc/fail2ban/jail.d/koprogo.conf
   
   [sshd]
   enabled = true
   port = ssh
   logpath = /var/log/auth.log
   maxretry = 3
   bantime = 3600
   findtime = 600
   
   [traefik-auth]
   enabled = true
   port = http,https
   logpath = /var/log/traefik/access.log
   maxretry = 5
   bantime = 1800
   findtime = 300
   filter = traefik-auth
   
   [traefik-badbots]
   enabled = true
   port = http,https
   logpath = /var/log/traefik/access.log
   maxretry = 2
   bantime = 86400
   findtime = 600
   filter = traefik-badbots
   
   [koprogo-api-abuse]
   enabled = true
   port = http,https
   logpath = /var/log/koprogo/backend.log
   maxretry = 20
   bantime = 3600
   findtime = 60
   filter = koprogo-api-abuse
   ```
   
   **Create filters:**
   
   `/etc/fail2ban/filter.d/traefik-auth.conf`:
   ```
   [Definition]
   failregex = ^.* (40[13]) .*$
   ignoreregex =
   ```
   
   `/etc/fail2ban/filter.d/traefik-badbots.conf`:
   ```
   [Definition]
   failregex = ^.* "(.*bot.*|.*crawler.*|.*spider.*)" .*$
   ignoreregex = (googlebot|bingbot|slackbot)
   ```
   
   `/etc/fail2ban/filter.d/koprogo-api-abuse.conf`:
   ```
   [Definition]
   failregex = Rate limit exceeded for IP: <HOST>
               Authentication failed for IP: <HOST>
   ignoreregex =
   ```
   
   **Deploy via Ansible:**
   ```yaml
   - name: Deploy fail2ban custom configuration
     template:
       src: fail2ban-koprogo.conf.j2
       dest: /etc/fail2ban/jail.d/koprogo.conf
     notify: restart fail2ban
   
   - name: Deploy fail2ban filters
     copy:
       src: "{{ item }}"
       dest: /etc/fail2ban/filter.d/
     with_items:
       - traefik-auth.conf
       - traefik-badbots.conf
       - koprogo-api-abuse.conf
     notify: restart fail2ban
   ```
   
   ### 2. Web Application Firewall (WAF) - Traefik Plugin
   
   **Option A: Traefik CrowdSec Plugin** (Recommended)
   
   Install CrowdSec bouncer for Traefik:
   ```bash
   # Install CrowdSec
   curl -s https://packagecloud.io/install/repositories/crowdsec/crowdsec/script.deb.sh | sudo bash
   apt install crowdsec crowdsec-firewall-bouncer-iptables
   
   # Install Traefik bouncer
   apt install crowdsec-traefik-bouncer
   ```
   
   **docker-compose.yml update:**
   ```yaml
   services:
     traefik:
       labels:
         - "traefik.http.middlewares.crowdsec.plugin.bouncer.enabled=true"
         - "traefik.http.middlewares.crowdsec.plugin.bouncer.crowdseclapikey=${CROWDSEC_API_KEY}"
   ```
   
   **Benefits:**
   - Shared threat intelligence from CrowdSec community
   - Automatic IP reputation blocking
   - Behavioral analysis
   - Easy integration with Traefik
   
   **Option B: ModSecurity WAF Rules** (Alternative)
   
   Use OWASP Core Rule Set (CRS) with custom Traefik integration.
   
   **Deploy via Ansible:**
   ```yaml
   - name: Install CrowdSec
     apt:
       name:
         - crowdsec
         - crowdsec-firewall-bouncer-iptables
         - crowdsec-traefik-bouncer
       state: present
   
   - name: Configure CrowdSec Traefik bouncer
     template:
       src: crowdsec-traefik-bouncer.yml.j2
       dest: /etc/crowdsec/bouncers/traefik-bouncer.yml
   ```
   
   ### 3. Intrusion Detection System (IDS)
   
   **Install Suricata** (lightweight IDS/IPS):
   
   ```bash
   apt install suricata
   ```
   
   **Enable Suricata rules:**
   ```yaml
   # /etc/suricata/suricata.yaml
   rule-files:
     - suricata.rules
     - emerging-threats.rules
     - local.rules
   
   # Custom rules for KoproGo
   alert http any any -> any any (msg:"SQL Injection Attempt"; content:"SELECT"; content:"FROM"; sid:1000001;)
   alert http any any -> any any (msg:"XSS Attempt"; content:"<script"; sid:1000002;)
   alert http any any -> any any (msg:"Path Traversal"; content:"../"; sid:1000003;)
   ```
   
   **Monitor Suricata alerts:**
   - Logs: `/var/log/suricata/fast.log`
   - Integration with monitoring stack (Loki)
   - Alert on critical events via Prometheus
   
   **Ansible task:**
   ```yaml
   - name: Install Suricata IDS
     apt:
       name: suricata
       state: present
   
   - name: Enable Suricata service
     systemd:
       name: suricata
       enabled: yes
       state: started
   
   - name: Deploy custom Suricata rules
     template:
       src: suricata-local.rules.j2
       dest: /etc/suricata/rules/local.rules
     notify: reload suricata
   ```
   
   ### 4. SSH Hardening
   
   **Current:** SSH enabled on port 22, password authentication allowed
   
   **Harden SSH configuration:**
   
   `/etc/ssh/sshd_config` updates:
   ```
   # Disable password authentication (key-only)
   PasswordAuthentication no
   PubkeyAuthentication yes
   PermitRootLogin prohibit-password
   
   # Disable empty passwords
   PermitEmptyPasswords no
   
   # Limit authentication attempts
   MaxAuthTries 3
   
   # Reduce login grace time
   LoginGraceTime 30
   
   # Restrict SSH protocol
   Protocol 2
   
   # Disable X11 forwarding
   X11Forwarding no
   
   # Enable strict mode
   StrictModes yes
   
   # Log verbosity
   LogLevel VERBOSE
   
   # Optional: Change SSH port (security through obscurity)
   # Port 2222
   ```
   
   **2FA via Google Authenticator (Optional):**
   ```bash
   apt install libpam-google-authenticator
   ```
   
   Update `/etc/pam.d/sshd`:
   ```
   auth required pam_google_authenticator.so
   ```
   
   **Ansible task:**
   ```yaml
   - name: Harden SSH configuration
     lineinfile:
       path: /etc/ssh/sshd_config
       regexp: "{{ item.regexp }}"
       line: "{{ item.line }}"
     with_items:
       - { regexp: '^PasswordAuthentication', line: 'PasswordAuthentication no' }
       - { regexp: '^PermitRootLogin', line: 'PermitRootLogin prohibit-password' }
       - { regexp: '^MaxAuthTries', line: 'MaxAuthTries 3' }
       - { regexp: '^LoginGraceTime', line: 'LoginGraceTime 30' }
     notify: restart sshd
   ```
   
   ### 5. Security Audit Automation
   
   **Install Lynis** (security auditing tool):
   ```bash
   apt install lynis
   ```
   
   **Schedule weekly security audits:**
   ```bash
   # Cron job: every Sunday at 3am
   0 3 * * 0 /usr/bin/lynis audit system --cronjob | tee /var/log/lynis/audit-$(date +\%Y\%m\%d).log
   ```
   
   **Parse Lynis results and alert on issues:**
   - Integration with monitoring stack
   - Alert if security score drops below threshold (e.g., 75/100)
   
   **Ansible task:**
   ```yaml
   - name: Install Lynis security auditing
     apt:
       name: lynis
       state: present
   
   - name: Schedule weekly security audit
     cron:
       name: "Lynis security audit"
       minute: "0"
       hour: "3"
       weekday: "0"
       job: "/usr/bin/lynis audit system --cronjob | tee /var/log/lynis/audit-$(date +\\%Y\\%m\\%d).log"
   ```
   
   ### 6. Additional Security Measures
   
   **A. Automatic Security Updates:**
   ```bash
   apt install unattended-upgrades
   dpkg-reconfigure -plow unattended-upgrades
   ```
   
   **B. Rootkit Detection (rkhunter):**
   ```bash
   apt install rkhunter
   rkhunter --update
   rkhunter --propupd
   ```
   
   Schedule daily scans:
   ```bash
   # Cron: daily at 4am
   0 4 * * * /usr/bin/rkhunter --check --skip-keypress --report-warnings-only
   ```
   
   **C. File Integrity Monitoring (AIDE):**
   ```bash
   apt install aide
   aideinit
   ```
   
   **D. Kernel Hardening (sysctl):**
   
   `/etc/sysctl.d/99-koprogo-hardening.conf`:
   ```
   # IP Forwarding
   net.ipv4.ip_forward = 0
   
   # SYN Cookies
   net.ipv4.tcp_syncookies = 1
   
   # Ignore ICMP redirects
   net.ipv4.conf.all.accept_redirects = 0
   net.ipv6.conf.all.accept_redirects = 0
   
   # Ignore source routed packets
   net.ipv4.conf.all.accept_source_route = 0
   
   # Log Martians
   net.ipv4.conf.all.log_martians = 1
   
   # Disable IPv6 (if not used)
   net.ipv6.conf.all.disable_ipv6 = 1
   ```
   
   Apply with: `sysctl -p`
   
   ## Testing & Validation
   
   - [ ] fail2ban custom jails block IPs after threshold
   - [ ] CrowdSec blocks known malicious IPs
   - [ ] Suricata detects SQL injection/XSS attempts (test with dummy payloads)
   - [ ] SSH login requires key only (password fails)
   - [ ] Lynis security score > 75/100
   - [ ] rkhunter detects no rootkits
   - [ ] AIDE baseline established
   - [ ] Security updates applied automatically
   
   ## Monitoring Integration
   
   - [ ] fail2ban metrics in Prometheus (`fail2ban_exporter`)
   - [ ] Suricata alerts in Loki
   - [ ] Lynis score tracked in Grafana
   - [ ] CrowdSec dashboard integrated
   - [ ] Alert on security score degradation
   
   ## Documentation
   
   - [ ] Update `infrastructure/README.md` with security procedures
   - [ ] Document fail2ban jail configurations
   - [ ] Create incident response playbook
   - [ ] Document SSH key management
   - [ ] Update CLAUDE.md with security posture
   
   ## Acceptance Criteria
   
   - [ ] fail2ban custom jails active (SSH, Traefik, API abuse)
   - [ ] CrowdSec WAF protecting Traefik endpoints
   - [ ] Suricata IDS monitoring network traffic
   - [ ] SSH hardened (key-only, reduced login grace time)
   - [ ] Weekly Lynis audits scheduled
   - [ ] Daily rkhunter scans scheduled
   - [ ] AIDE file integrity monitoring active
   - [ ] Kernel hardened via sysctl
   - [ ] Automatic security updates enabled
   - [ ] Monitoring dashboards show security metrics
   - [ ] Documentation complete
   
   ## Resource Impact
   
   - CrowdSec: ~50MB RAM
   - Suricata: ~100MB RAM
   - Lynis/rkhunter: Cron jobs, minimal overhead
   - **Total: ~150MB RAM additional**
   
   ## Effort Estimate
   
   **Medium** (2 days)
   - Day 1: fail2ban jails + CrowdSec WAF + SSH hardening
   - Day 2: Suricata IDS + security audit tools + testing
   
   ## Related
   
   - Supports: Production security posture
   - Integrates with: Issue #41 (monitoring stack)
   - Complements: Issue #39 (encryption at rest)
   
   ## References
   
   - fail2ban: https://www.fail2ban.org/
   - CrowdSec: https://www.crowdsec.net/
   - Suricata: https://suricata.io/
   - Lynis: https://cisofy.com/lynis/
   - OWASP WAF: https://owasp.org/www-project-modsecurity-core-rule-set/

.. raw:: html

   </div>

