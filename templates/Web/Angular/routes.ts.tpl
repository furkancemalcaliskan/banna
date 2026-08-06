import { authGuard, permissionGuard } from '@abp/ng.core';
import { Routes } from '@angular/router';

export const __entity_name_lower__Routes: Routes = [
  {
    path: '',
    pathMatch: 'full',
    canActivate: [authGuard, permissionGuard],
    data: { requiredPolicy: '__domain_short__.__entity_plural__' },
    loadComponent: () => import('./__entity_name_kebab__.component').then(m => m.__entity_name__Component),
  },
];
