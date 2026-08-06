import api from './API';

type ListParams = {
  filter?: string;
  maxResultCount?: number;
  skipCount?: number;
};

export const getList = ({ filter, maxResultCount, skipCount }: ListParams = {}) =>
  api
    .get('/api/app/__entity_plural_kebab__', {
      params: {
        filterText: filter,
        maxResultCount,
        skipCount,
      },
    })
    .then(({ data }) => data);

export const get = (id: string | number) =>
  api.get('/api/app/__entity_plural_kebab__/' + id).then(({ data }) => data);

export const create = (input: Record<string, any>) =>
  api.post('/api/app/__entity_plural_kebab__', input).then(({ data }) => data);

export const update = (input: Record<string, any>, id: string | number) =>
  api.put('/api/app/__entity_plural_kebab__/' + id, input).then(({ data }) => data);

export const remove = (id: string | number) =>
  api.delete('/api/app/__entity_plural_kebab__/' + id).then(({ data }) => data);

__mobile_lookup_methods__
