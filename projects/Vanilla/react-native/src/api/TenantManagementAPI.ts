import api from './API';

export function getTenants(params: Record<string, any> = {}) {
  return api.get('/api/multi-tenancy/tenants', { params }).then(({ data }) => data);
}

export function createTenant(body: any) {
  return api.post('/api/multi-tenancy/tenants', body).then(({ data }) => data);
}

export function getTenantById(id: string) {
  return api.get(`/api/multi-tenancy/tenants/${id}`).then(({ data }) => data);
}

export function updateTenant(body: any, id: string) {
  return api.put(`/api/multi-tenancy/tenants/${id}`, body).then(({ data }) => data);
}

export function removeTenant(id: string) {
  return api.delete(`/api/multi-tenancy/tenants/${id}`).then(({ data }) => data);
}
