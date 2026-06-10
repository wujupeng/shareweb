import api from './index'

export const adminApi = {
  listUsers(params?: { status?: string; page?: number; page_size?: number }) {
    return api.get('/users', { params })
  },
  createUser(data: { username: string; password: string; role?: string }) {
    return api.post('/users', data)
  },
  updateUser(username: string, data: { role?: string; status?: string; password?: string }) {
    return api.put(`/users/${username}`, data)
  },
  deleteUser(username: string) {
    return api.delete(`/users/${username}`)
  },
  listPermissions(params?: { path?: string }) {
    return api.get('/permissions', { params })
  },
  createPermission(data: { path: string; role: string; allowed_actions: string[]; inherit?: boolean }) {
    return api.post('/permissions', data)
  },
  deletePermission(id: number) {
    return api.delete(`/permissions/${id}`)
  },
  listAuditLogs(params?: { operator?: string; action_type?: string; start_time?: string; end_time?: string; page?: number; page_size?: number }) {
    return api.get('/audit-logs', { params })
  },
}
