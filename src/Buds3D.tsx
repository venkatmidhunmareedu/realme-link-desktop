import { useEffect, useRef, useState } from "react";
import gsap from "gsap";
import * as THREE from "three";
import { RoomEnvironment } from "three/examples/jsm/environments/RoomEnvironment.js";
import { GLTFLoader } from "three/examples/jsm/loaders/GLTFLoader.js";

const SIGNAL = "#ffd60a";

/**
 * Procedural earbud pair. If `/models/<slug>.glb` exists (drop official or custom assets there)
 * it replaces the procedural model for that device.
 */
function procedural(): { root: THREE.Group; buds: THREE.Group[] } {
  const root = new THREE.Group();
  const shell = new THREE.MeshPhysicalMaterial({ color: 0xf4f1ea, roughness: 0.22, clearcoat: 1, clearcoatRoughness: 0.12 });
  const trim = new THREE.MeshPhysicalMaterial({ color: SIGNAL, roughness: 0.3, metalness: 0.6, clearcoat: 0.6 });
  const mesh = new THREE.MeshPhysicalMaterial({ color: 0x15161b, roughness: 0.7 });

  const bud = (side: 1 | -1) => {
    const b = new THREE.Group();
    const head = new THREE.Mesh(new THREE.SphereGeometry(0.34, 48, 48), shell);
    head.scale.set(1, 0.92, 1.05);
    const nozzle = new THREE.Mesh(new THREE.CylinderGeometry(0.11, 0.15, 0.26, 32), mesh);
    nozzle.rotation.z = (Math.PI / 2) * -side;
    nozzle.position.set(-0.38 * side, -0.02, 0.02);
    const stem = new THREE.Mesh(new THREE.CapsuleGeometry(0.085, 0.62, 12, 24), shell);
    stem.position.set(0.1 * side, -0.52, -0.06);
    stem.rotation.z = 0.12 * side;
    const ring = new THREE.Mesh(new THREE.TorusGeometry(0.2, 0.014, 16, 64), trim);
    ring.position.set(0.12 * side, 0.02, 0.27);
    b.add(head, nozzle, stem, ring);
    b.position.set(0.78 * side, 0.05, 0);
    b.rotation.set(0.1, 0.35 * side, -0.18 * side);
    return b;
  };
  const buds = [bud(-1), bud(1)];
  root.add(...buds);
  return { root, buds };
}

export function Buds3D({ slug }: { slug: string }) {
  const host = useRef<HTMLDivElement>(null);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const el = host.current!;
    const reduce = matchMedia("(prefers-reduced-motion: reduce)").matches;
    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.05;
    el.appendChild(renderer.domElement);

    const scene = new THREE.Scene();
    scene.environment = new THREE.PMREMGenerator(renderer).fromScene(new RoomEnvironment(), 0.04).texture;
    const cam = new THREE.PerspectiveCamera(32, 1, 0.1, 50);
    cam.position.set(0, 0.15, 5.4);
    const key = new THREE.DirectionalLight(0xffffff, 1.4);
    key.position.set(2, 3, 4);
    scene.add(key);

    const pivot = new THREE.Group();
    scene.add(pivot);
    const state = { spin: -0.5, vel: 0.0035, drag: false, lastX: 0 };

    /** The one orchestrated moment: parts fly in and settle, then the idle spin takes over. */
    const assemble = (parts: THREE.Object3D[], target: THREE.Object3D) => {
      setReady(true);
      if (reduce) return;
      const tl = gsap.timeline();
      tl.from(renderer.domElement, { opacity: 0, duration: 0.6, ease: "power2.out" }, 0);
      tl.from(state, { spin: state.spin - 3.2, vel: 0.05, duration: 1.8, ease: "power3.out" }, 0);
      parts.forEach((p, i) => {
        const home = p.position.clone();
        tl.from(p.position, { x: home.x + (i ? 2.6 : -2.6), y: home.y - 0.6, duration: 1.1, ease: "expo.out" }, 0.08 + i * 0.12);
        tl.from(p.scale, { x: 0.01, y: 0.01, z: 0.01, duration: 0.9, ease: "back.out(1.6)" }, 0.08 + i * 0.12);
      });
      if (!parts.length) tl.from(target.scale, { x: 0.01, y: 0.01, z: 0.01, duration: 1, ease: "back.out(1.5)" }, 0);
    };

    const fallback = procedural();
    let model: THREE.Object3D = fallback.root;
    pivot.add(model);
    new GLTFLoader().load(
      `/models/${slug}.glb`,
      (gltf) => {
        pivot.remove(model);
        model = gltf.scene;
        const box = new THREE.Box3().setFromObject(model);
        const k = 2.6 / box.getSize(new THREE.Vector3()).length();
        model.scale.setScalar(k);
        model.position.sub(box.getCenter(new THREE.Vector3()).multiplyScalar(k));
        pivot.add(model);
        assemble([], model);
      },
      undefined,
      () => assemble(fallback.buds, fallback.root) // no asset for this model: animate the procedural pair
    );

    const down = (e: PointerEvent) => ((state.drag = true), (state.lastX = e.clientX));
    const up = () => (state.drag = false);
    const move = (e: PointerEvent) => {
      if (!state.drag) return;
      state.vel = (e.clientX - state.lastX) * 0.006;
      state.spin += state.vel;
      state.lastX = e.clientX;
    };
    el.addEventListener("pointerdown", down);
    addEventListener("pointerup", up);
    addEventListener("pointermove", move);

    const resize = () => {
      const { clientWidth: w, clientHeight: h } = el;
      if (!w || !h) return;
      renderer.setSize(w, h);
      cam.aspect = w / h;
      // keep the whole pair in frame whatever the panel's shape
      cam.position.z = Math.max(5.4, 1.9 / (Math.tan((cam.fov * Math.PI) / 360) * cam.aspect));
      cam.updateProjectionMatrix();
    };
    const ro = new ResizeObserver(resize);
    ro.observe(el);
    resize();

    let raf = 0;
    const clock = new THREE.Clock();
    const tick = () => {
      const t = clock.getElapsedTime();
      if (!state.drag) {
        state.spin += state.vel;
        state.vel += (0.0035 - state.vel) * 0.04; // ease back to a slow idle spin
      }
      pivot.rotation.y = state.spin;
      pivot.position.y = Math.sin(t * 0.9) * 0.05;
      renderer.render(scene, cam);
      raf = requestAnimationFrame(tick);
    };
    tick();

    return () => {
      cancelAnimationFrame(raf);
      gsap.killTweensOf([state, renderer.domElement]);
      ro.disconnect();
      el.removeEventListener("pointerdown", down);
      removeEventListener("pointerup", up);
      removeEventListener("pointermove", move);
      renderer.dispose();
      el.removeChild(renderer.domElement);
      setReady(false);
    };
  }, [slug]);

  return (
    <div ref={host} className="relative h-full min-h-72 w-full cursor-grab active:cursor-grabbing" aria-label="Interactive 3D model of your earbuds" role="img">
      {!ready && <div className="absolute inset-0 grid place-items-center text-sm text-muted-foreground">Loading model…</div>}
    </div>
  );
}
