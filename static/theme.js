;(() => {
  const key = 'quizinart-theme'

  const getPreferredTheme = () => {
    const saved = localStorage.getItem(key)
    if (saved === 'light' || saved === 'dark' || saved === 'system') {
      return saved
    }
    return 'system'
  }

  const resolveTheme = (mode) => {
    if (mode === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light'
    }
    return mode
  }

  const applyTheme = (mode) => {
    const theme = resolveTheme(mode)
    document.documentElement.setAttribute('data-theme', theme)
    document.documentElement.setAttribute('data-theme-mode', mode)
  }

  const mode = getPreferredTheme()
  applyTheme(mode)

  document.addEventListener('DOMContentLoaded', () => {
    const select = document.querySelector('.theme-select')
    if (!select) {
      return
    }

    select.value = mode
    select.addEventListener('change', (event) => {
      const selectedMode = event.target.value
      localStorage.setItem(key, selectedMode)
      applyTheme(selectedMode)
    })
  })

  window
    .matchMedia('(prefers-color-scheme: dark)')
    .addEventListener('change', () => {
      const currentMode = getPreferredTheme()
      if (currentMode === 'system') {
        applyTheme(currentMode)
      }
    })
})()
