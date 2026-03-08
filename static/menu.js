;(() => {
  document.addEventListener('DOMContentLoaded', () => {
    const navBtn = document.getElementById('nav-toggle')
    const navMenu = document.getElementById('nav-menu')
    const settingsBtn = document.getElementById('settings-toggle')
    const settingsMenu = document.getElementById('settings-menu')

    function closeNav() {
      if (navMenu) {
        navMenu.classList.remove('open')
        navBtn?.setAttribute('aria-expanded', 'false')
      }
    }

    function closeSettings() {
      if (settingsMenu) {
        settingsMenu.classList.remove('open')
        settingsBtn?.setAttribute('aria-expanded', 'false')
      }
    }

    if (navBtn && navMenu) {
      navBtn.addEventListener('click', (e) => {
        e.stopPropagation()
        closeSettings()
        const open = navMenu.classList.toggle('open')
        navBtn.setAttribute('aria-expanded', open)
      })
    }

    if (settingsBtn && settingsMenu) {
      settingsBtn.addEventListener('click', (e) => {
        e.stopPropagation()
        closeNav()
        const open = settingsMenu.classList.toggle('open')
        settingsBtn.setAttribute('aria-expanded', open)
      })

      settingsMenu.addEventListener('click', (e) => {
        e.stopPropagation()
      })
    }

    document.addEventListener('click', () => {
      closeNav()
      closeSettings()
    })

    document.addEventListener('keydown', (e) => {
      if (e.key !== 'Escape') return
      if (navMenu?.classList.contains('open')) {
        closeNav()
        navBtn?.focus()
      }
      if (settingsMenu?.classList.contains('open')) {
        closeSettings()
        settingsBtn?.focus()
      }
    })
  })
})()
