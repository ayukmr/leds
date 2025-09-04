import * as THREE from 'three';
import * as monaco from 'monaco-editor';

import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { OBJLoader } from 'three/addons/loaders/OBJLoader.js';

const code = `let strength = noise(time, height * 5 + time * 2, 0.5) ** 2.5;
strength += height * 1.2 - 0.5;

const r = clamp(strength * 2 - 0.5) * 255;
const g = clamp(strength * 4 - 3) * 255;
const b = clamp(strength * 4);

return [r, g, b];`;

let cam, scene, rnd, controls, strips;

function clamp(strength) {
  return Math.max(0, Math.min(1, strength))
}

const Noise = new function(){function r(r){return r*r*r*(r*(6*r-15)+10)}function o(r,o,n){return o+r*(n-o)}function n(r,o,n,t){var f=15&r,a=f<8?o:n,u=f<4?n:12==f||14==f?o:t;return(1&f?-a:a)+(2&f?-u:u)}this.noise=function(t,f,a){var u=new Array(512),e=[151,160,137,91,90,15,131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,190,6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,88,237,149,56,87,174,20,125,136,171,168,68,175,74,165,71,134,139,48,27,166,77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,102,143,54,65,25,63,161,1,216,80,73,209,76,132,187,208,89,18,169,200,196,135,130,116,188,159,86,164,100,109,198,173,186,3,64,52,217,226,250,124,123,5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,223,183,170,213,119,248,152,2,44,154,163,70,221,153,101,155,167,43,172,9,129,22,39,253,19,98,108,110,79,113,224,232,178,185,112,104,218,246,97,228,251,34,242,193,238,210,144,12,191,179,162,241,81,51,145,235,249,14,239,107,49,192,214,31,181,199,106,157,184,84,204,176,115,121,50,45,127,4,150,254,138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180];for(let r=0;r<256;r++)u[256+r]=u[r]=e[r];var h=255&Math.floor(t),i=255&Math.floor(f),l=255&Math.floor(a);t-=Math.floor(t),f-=Math.floor(f),a-=Math.floor(a);var M=r(t),c=r(f),v=r(a),s=u[h]+i,w=u[s]+l,y=u[s+1]+l,A=u[h+1]+i,b=u[A]+l,d=u[A+1]+l;return(1+o(v,o(c,o(M,n(u[w],t,f,a),n(u[b],t-1,f,a)),o(M,n(u[y],t,f-1,a),n(u[d],t-1,f-1,a))),o(c,o(M,n(u[w+1],t,f,a-1),n(u[b+1],t-1,f,a-1)),o(M,n(u[y+1],t,f-1,a-1),n(u[d+1],t-1,f-1,a-1)))))/2}};

let sample = new Function('height', 'time', 'clamp', 'noise', code);
let time = 0;

initScene();
initEditor();

load('model.obj');

setInterval(() => update(), 1000/60);

function initScene() {
  rnd = new THREE.WebGLRenderer({ antialias: true });

  rnd.setPixelRatio(window.devicePixelRatio);
  rnd.setSize(window.innerWidth * 0.55, window.innerHeight);

  rnd.outputColorSpace = THREE.SRGBColorSpace;
  rnd.toneMapping = THREE.ACESFilmicToneMapping;

  document.body.appendChild(rnd.domElement);

  scene = new THREE.Scene();
  scene.background = new THREE.Color(0x1a1a1a);

  cam = new THREE.PerspectiveCamera(
    45,
    window.innerWidth / window.innerHeight,
    0.1,
    5000
  );

  cam.aspect = window.innerWidth * 0.55 / window.innerHeight;
  cam.position.set(2, 2, 2);

  controls = new OrbitControls(cam, rnd.domElement);
  controls.enableDamping = true;

  scene.add(new THREE.HemisphereLight(0xffffff, 0x555555, 1.0));

  rnd.setAnimationLoop(() => {
    controls.update();
    rnd.render(scene, cam);
  });

  window.addEventListener('resize', resize);
}

function brightenMats(root) {
  const toArray = (m) => (Array.isArray(m) ? m : [m]);

  root.traverse((child) => {
    if (!child.isMesh || !child.material) return;

    const mats = toArray(child.material).map((m) => m.clone());
    child.material = mats.length === 1 ? mats[0] : mats;

    for (const m of mats) {
      m.emissive.lerp(new THREE.Color(0xffffff), 0.05);
      m.emissiveIntensity = Math.max(m.emissiveIntensity ?? 0, 0.2);
    }

    child.geometry.computeVertexNormals();
  });
}

function addEdges(root) {
  root.traverse((child) => {
    if (!child.isMesh || !child.geometry) return;

    const edges = new THREE.EdgesGeometry(child.geometry, 25);

    const mat = new THREE.LineBasicMaterial({
      color: 0x000000,
      transparent: true,
      opacity: 0.5
    });

    const lines = new THREE.LineSegments(edges, mat);
    lines.matrixAutoUpdate = false;

    child.add(lines);
  });
}

function resize() {
  cam.aspect = window.innerWidth * 0.55 / window.innerHeight;
  cam.updateProjectionMatrix();

  rnd.setSize(window.innerWidth * 0.55, window.innerHeight);
}

function initEditor() {
  const editor = monaco.editor.create(
    document.getElementById('editor'),
    {
      value: code,
      language: 'javascript',
      theme: 'vs-dark',
      lineNumbers: 'off',
      folding: false,
      scrollBeyondLastLine: false,
      minimap: { enabled: false },
      stickyScroll: { enabled: false },
      scrollbar: { useShadows: false }
    }
  );

  document.getElementById('reload')
    .addEventListener('click', () => {
      sample = new Function('height', 'time', 'clamp', 'noise', editor.getValue());
    });
}

function load(path) {
  new OBJLoader().load(path, (obj) => {
    scene.add(obj);

    brightenMats(obj);
    addEdges(obj);

    const box = new THREE.Box3().setFromObject(obj);

    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());

    obj.position.sub(center);

    const fit = Math.max(size.x, size.y, size.z) || 1;
    const dist = fit / (2 * Math.tan(THREE.MathUtils.degToRad(cam.fov) / 2)) * 1.25;

    cam.position.set(-dist, 0, dist);
    controls.target.set(0, 0, 0);

    cam.near = Math.max(0.01, fit / 1000);
    cam.far = Math.max(1000, fit * 10);
    cam.updateProjectionMatrix();

    createStrips();
  });
}

function createStrips() {
  strips = [1, -1].map((x) => {
    const a = sampleArray(40);
    const b = sampleArray(24);

    return [
      createStrip(
        23,
        a,
        new THREE.Vector3(0.385, -2.3, 10.5 * x),
        new THREE.Euler(
          THREE.MathUtils.degToRad(90),
          THREE.MathUtils.degToRad(-61.5),
          0
        )
      ),
      createStrip(
        4.6,
        b.slice(0, 8).reverse(),
        new THREE.Vector3(-5.1, 5.5, 10.5 * x),
        new THREE.Euler(
          THREE.MathUtils.degToRad(90),
          THREE.MathUtils.degToRad(90),
          0
        )
      ),
      createStrip(
        9,
        b.slice(8, 24),
        new THREE.Vector3(-2.95, -0.75, 10.5 * x),
        new THREE.Euler(
          THREE.MathUtils.degToRad(90),
          THREE.MathUtils.degToRad(-61.5),
          0
        )
      )
    ];
  });
}

function createStrip(w, colors, pos, rot) {
  const n = colors.length;
  const sw = w / n;

  const group = new THREE.Group();

  colors.forEach((c, i) => {
    const geo = new THREE.BoxGeometry(sw, 0.5, 0.1);
    const mat = new THREE.MeshStandardMaterial({
      color: c,
      emissive: c,
      emissiveIntensity: 2
    });

    const box = new THREE.Mesh(geo, mat);
    box.position.set(-w / 2 + sw / 2 + i * sw, 0, 0);

    group.add(box);
  });

  group.position.copy(pos);
  group.rotation.copy(rot);

  scene.add(group);

  return group;
}

function update() {
  if (!strips) return;

  time += 1/60;

  for (const side of strips) {
    const a = sampleArray(40);
    const b = sampleArray(24);

    applyColors(a, side[0]);
    applyColors(b.slice(0, 8).reverse(), side[1]);
    applyColors(b.slice(8, 24), side[2]);
  }
}

function sampleArray(h) {
  const colors = [];

  for (let i = 0; i < h; i++) {
    const [r, g, b] = sample(i / (h - 1), time, clamp, Noise.noise);
    const hex = (r & 0xff) << 16 | (g & 0xff) << 8 | (b & 0xff);

    colors.push(hex);
  }

  return colors;
}

function applyColors(colors, strip) {
  colors.forEach((c, i) => {
    const mat = new THREE.MeshStandardMaterial({
      color: c,
      emissive: c,
      emissiveIntensity: 2
    });

    strip.children[i].material = mat;
  });
}
