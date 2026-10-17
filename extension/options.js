import { getHost, setHost } from './modules/config.js';

const input = document.getElementById('host');
const status = document.getElementById('status');

input.value = await getHost();

document.getElementById('save').addEventListener('click', async () => {
  try {
    input.value = await setHost(input.value);
    const { protocol, hostname } = new URL(input.value);
    const granted = await chrome.permissions.request({ origins: [`${protocol}//${hostname}/*`] });
    status.textContent = granted ? 'Saved.' : 'Saved, but the browser did not grant access to that address.';
  } catch (error) {
    status.textContent = error.message;
  }
});
