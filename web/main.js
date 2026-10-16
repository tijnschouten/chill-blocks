import init, { boot } from './pkg/chillblocks_app.js';

async function start() {
  await init();
  boot();
}

start().catch((error) => {
  const element = document.getElementById('error');
  element.hidden = false;
  element.textContent = `ChillBlocks could not start: ${error}`;
});
