$(function () {

    const iconEl = document.getElementById('ThemeIcon');

    function setIcon(theme) {
        if (theme === 'dark') {
            iconEl.classList.remove('fa-sun');
            iconEl.classList.add('fa-moon');
        } else {
            iconEl.classList.remove('fa-moon');
            iconEl.classList.add('fa-sun');
        }
    }

    function changeTheme(theme) {
        window.localStorage.setItem('theme', theme);
        document.body.setAttribute('data-bs-theme', theme);
        setIcon(theme);
    }

    function toggleTheme() {
        getTheme() === 'light' ? changeTheme('dark') : changeTheme('light');
    }

    function getTheme() {
        return window.localStorage.getItem('theme') ?? 'dark';
    }

    function init() {
        const theme = getTheme();
        changeTheme(theme);
    }

    document
        .getElementById('ToolbarChangeTheme')
        .addEventListener('click', toggleTheme);

    init();
});