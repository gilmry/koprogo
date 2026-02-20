<script lang="ts">
  import { authStore } from "../stores/auth";
  import { UserRole } from "../lib/types";

  $: user = $authStore.user;

  function getRoleLabel(role: UserRole | undefined): string {
    switch (role) {
      case UserRole.SUPERADMIN: return "Administrateur plateforme";
      case UserRole.SYNDIC: return "Syndic";
      case UserRole.ACCOUNTANT: return "Comptable";
      case UserRole.OWNER: return "Copropriétaire";
      default: return "Utilisateur";
    }
  }

  const settingsSections = [
    {
      href: "/profile",
      icon: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z",
      color: "bg-amber-50 border-amber-200 text-amber-600",
      badge: "amber",
      title: "Profil & Informations personnelles",
      description: "Modifier votre nom, prénom, email et consulter vos rôles.",
      label: "Gérer le profil",
    },
    {
      href: "/settings/notifications",
      icon: "M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9",
      color: "bg-blue-50 border-blue-200 text-blue-600",
      badge: "blue",
      title: "Notifications",
      description: "Gérer les canaux Email, SMS, Push et In-App pour chaque type d'événement.",
      label: "Gérer les notifications",
    },
    {
      href: "/settings/gdpr",
      icon: "M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z",
      color: "bg-green-50 border-green-200 text-green-600",
      badge: "green",
      title: "RGPD — Mes données personnelles",
      description: "Exporter, rectifier, effacer ou restreindre le traitement de vos données (Art. 15-21).",
      label: "Gérer mes données",
    },
  ];
</script>

<div class="space-y-6">
  {#if user}
    <!-- Profile summary banner -->
    <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-5 flex items-center gap-4">
      <div class="w-12 h-12 rounded-full bg-amber-100 flex items-center justify-center shrink-0">
        <span class="text-amber-700 font-bold text-lg">
          {(user.first_name?.[0] ?? user.email[0]).toUpperCase()}
        </span>
      </div>
      <div class="flex-1 min-w-0">
        <p class="font-semibold text-gray-900 truncate">
          {user.first_name} {user.last_name}
        </p>
        <p class="text-sm text-gray-500 truncate">{user.email}</p>
        <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-800 mt-1">
          {getRoleLabel(user.role)}
        </span>
      </div>
      <a
        href="/profile"
        class="shrink-0 text-sm text-primary-600 hover:text-primary-700 font-medium"
      >
        Voir le profil →
      </a>
    </div>
  {/if}

  <!-- Settings navigation cards -->
  <div class="space-y-3">
    {#each settingsSections as section}
      <a
        href={section.href}
        class="block bg-white rounded-lg border {section.color.split(' ')[1]} hover:shadow-md transition-shadow p-5 group"
      >
        <div class="flex items-start gap-4">
          <div class="shrink-0 w-10 h-10 rounded-lg {section.color.split(' ')[0]} flex items-center justify-center">
            <svg class="w-5 h-5 {section.color.split(' ')[2]}" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={section.icon}/>
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <p class="font-semibold text-gray-900 group-hover:text-primary-700 transition-colors">
              {section.title}
            </p>
            <p class="text-sm text-gray-500 mt-0.5">{section.description}</p>
          </div>
          <svg class="w-5 h-5 text-gray-400 group-hover:text-primary-500 transition-colors shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
          </svg>
        </div>
      </a>
    {/each}
  </div>

  <!-- Upcoming features -->
  <div class="bg-gray-50 rounded-lg border border-gray-200 p-5">
    <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wider mb-3">
      Fonctionnalités à venir
    </h3>
    <ul class="space-y-2 text-sm text-gray-500">
      <li class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full bg-gray-300"></span>
        Langue de l'interface (Français, English, Nederlands)
      </li>
      <li class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full bg-gray-300"></span>
        Authentification à deux facteurs (2FA)
      </li>
      <li class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full bg-gray-300"></span>
        Changement de mot de passe
      </li>
      <li class="flex items-center gap-2">
        <span class="w-1.5 h-1.5 rounded-full bg-gray-300"></span>
        Intégrations (API, SMTP, services cloud)
      </li>
    </ul>
  </div>
</div>
