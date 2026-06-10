import { defineStore } from 'pinia'
import { ref } from 'vue'
import { authApi } from '@/api/auth'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string>(localStorage.getItem('token') || '')
  const username = ref<string>(localStorage.getItem('username') || '')
  const role = ref<string>(localStorage.getItem('role') || '')

  async function login(user: string, password: string) {
    const res = await authApi.login({ username: user, password })
    token.value = res.data.data.token
    username.value = user
    role.value = res.data.data.role
    localStorage.setItem('token', token.value)
    localStorage.setItem('username', username.value)
    localStorage.setItem('role', role.value)
  }

  function logout() {
    token.value = ''
    username.value = ''
    role.value = ''
    localStorage.removeItem('token')
    localStorage.removeItem('username')
    localStorage.removeItem('role')
  }

  const isAdmin = () => role.value === 'admin'
  const isReadWrite = () => role.value === 'readwrite' || role.value === 'admin'
  const isLoggedIn = () => !!token.value

  return { token, username, role, login, logout, isAdmin, isReadWrite, isLoggedIn }
})
