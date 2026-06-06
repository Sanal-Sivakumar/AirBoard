// --- Intersection Observer for Scroll Animations ---
document.addEventListener('DOMContentLoaded', () => {
    const fadeElements = document.querySelectorAll('.fade-in, .reveal-text');
    
    const observerOptions = {
        root: null,
        threshold: 0.15,
        rootMargin: "0px 0px -40px 0px"
    };
    
    const observer = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.classList.add('active');
                if (entry.target.classList.contains('fade-in')) {
                    observer.unobserve(entry.target); // Fade-in elements trigger once
                }
            } else {
                if (entry.target.classList.contains('reveal-text')) {
                    entry.target.classList.remove('active'); // Text reveal can trigger on/off
                }
            }
        });
    }, observerOptions);
    
    fadeElements.forEach(el => observer.observe(el));
});


// --- Three.js Cinematic 3D Loop & Data Flow System ---
let scene, camera, renderer;
let torusKnot, particleSystem, particleGeometry;
let particlesData = [];
let targetMouseX = 0, targetMouseY = 0;
let currentMouseX = 0, currentMouseY = 0;
let scrollPercent = 0;

const PARTICLE_COUNT = 450;
const P_KNOT = 2; // Torus knot configuration p
const Q_KNOT = 3; // Torus knot configuration q
const SCALE_KNOT = 1.35; // Size multiplier for coordinates

// Get coordinate points on the Torus Knot path
function getTorusKnotPoint(t) {
    const r = (2.0 + Math.cos(Q_KNOT * t)) * SCALE_KNOT;
    const x = r * Math.cos(P_KNOT * t);
    const y = r * Math.sin(P_KNOT * t);
    const z = Math.sin(Q_KNOT * t) * SCALE_KNOT;
    return new THREE.Vector3(x, y, z);
}

// Initialize Three.js WebGL scene
function init3D() {
    const canvas = document.getElementById('three-canvas');
    if (!canvas) return;

    // Scene setup
    scene = new THREE.Scene();
    scene.fog = new THREE.FogExp2(0x000000, 0.02);

    // Camera setup
    camera = new THREE.PerspectiveCamera(55, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.z = 7.5;

    // Renderer setup
    renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.3;

    // 1. Create central glass Infinity Torus Knot
    const knotGeom = new THREE.TorusKnotGeometry(2, 0.42, 200, 32, P_KNOT, Q_KNOT);
    const knotMat = new THREE.MeshPhysicalMaterial({
        color: 0x00f2fe,
        transparent: true,
        opacity: 0.18,
        transmission: 0.92,
        roughness: 0.05,
        metalness: 0.05,
        ior: 1.75,
        thickness: 2.2,
        depthWrite: false
    });
    torusKnot = new THREE.Mesh(knotGeom, knotMat);
    scene.add(torusKnot);

    // Add glowing wireframe edges
    const edges = new THREE.EdgesGeometry(knotGeom);
    const lineMat = new THREE.LineBasicMaterial({
        color: 0x0071e3,
        transparent: true,
        opacity: 0.25
    });
    const wireframe = new THREE.LineSegments(edges, lineMat);
    torusKnot.add(wireframe);

    // 2. Create flowing particle data stream along the Knot path
    particleGeometry = new THREE.BufferGeometry();
    const positions = new Float32Array(PARTICLE_COUNT * 3);
    const colors = new Float32Array(PARTICLE_COUNT * 3);

    const color1 = new THREE.Color(0x00f2fe); // Cyan
    const color2 = new THREE.Color(0x7f00ff); // Purple

    for (let i = 0; i < PARTICLE_COUNT; i++) {
        // Space particles uniformly along the 2pi path loop
        const t = (i / PARTICLE_COUNT) * Math.PI * 2;
        const point = getTorusKnotPoint(t);
        
        positions[i * 3] = point.x;
        positions[i * 3 + 1] = point.y;
        positions[i * 3 + 2] = point.z;

        // Gradient coloring from cyan to purple along the loop path
        const mixRatio = i / PARTICLE_COUNT;
        const mixedColor = color1.clone().lerp(color2, mixRatio);
        
        colors[i * 3] = mixedColor.r;
        colors[i * 3 + 1] = mixedColor.g;
        colors[i * 3 + 2] = mixedColor.b;

        // Store individual particle progress metadata (angle position & velocity speed)
        particlesData.push({
            t: t,
            speed: 0.003 + Math.random() * 0.005
        });
    }

    particleGeometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    particleGeometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));

    // Particle mesh material
    const particleMat = new THREE.PointsMaterial({
        size: 0.09,
        vertexColors: true,
        transparent: true,
        opacity: 0.85,
        blending: THREE.AdditiveBlending,
        depthWrite: false
    });

    particleSystem = new THREE.Points(particleGeometry, particleMat);
    scene.add(particleSystem);

    // 3. Add Studio lighting
    addLighting();

    // Event hooks
    window.addEventListener('resize', onWindowResize);
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('scroll', onWindowScroll);

    // Start Rendering Loop
    animate();
}

function addLighting() {
    // Soft Ambient
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.35);
    scene.add(ambientLight);

    // Spot Cyan Glow from top-left
    const spotCyan = new THREE.SpotLight(0x00f2fe, 10, 30, Math.PI / 4, 0.6, 1);
    spotCyan.position.set(-8, 10, 8);
    scene.add(spotCyan);

    // Spot Purple Glow from bottom-right
    const spotPurple = new THREE.SpotLight(0x7f00ff, 8, 30, Math.PI / 4, 0.6, 1);
    spotPurple.position.set(8, -10, -6);
    scene.add(spotPurple);

    // Dynamic backlight to create glass refraction highlights
    const backLight = new THREE.DirectionalLight(0xffffff, 1.5);
    backLight.position.set(0, 0, -10);
    scene.add(backLight);
}

// Mouse Move Parallax coordinates
function onMouseMove(event) {
    targetMouseX = (event.clientX - window.innerWidth / 2) / (window.innerWidth / 2);
    targetMouseY = (event.clientY - window.innerHeight / 2) / (window.innerHeight / 2);
}

// Calculate scroll depth percentage
function onWindowScroll() {
    const totalHeight = document.documentElement.scrollHeight - window.innerHeight;
    if (totalHeight > 0) {
        scrollPercent = window.scrollY / totalHeight;
    }
}

// Handle window resizing scale
function onWindowResize() {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
}

// Rendering Animation loop
function animate() {
    requestAnimationFrame(animate);

    // 1. Slow, liquid glass morphing & rotation for the Torus Knot
    if (torusKnot) {
        torusKnot.rotation.y += 0.005;
        torusKnot.rotation.x = Math.sin(Date.now() * 0.0004) * 0.12;
        torusKnot.rotation.z = Date.now() * 0.0001;

        // Apply a gentle morphing oscillation to the torus shape scale
        const morphScale = 1 + Math.sin(Date.now() * 0.0008) * 0.04;
        torusKnot.scale.set(morphScale, morphScale, morphScale);
    }

    // 2. Animate and flow the particle stream along the Torus Knot mathematics
    if (particleSystem && particleGeometry) {
        const positions = particleGeometry.attributes.position.array;

        for (let i = 0; i < PARTICLE_COUNT; i++) {
            const data = particlesData[i];
            
            // Advance particle progress along loop
            data.t += data.speed;
            if (data.t > Math.PI * 2) {
                data.t -= Math.PI * 2; // Loop back around
            }

            // Calculate new 3D location coordinate
            const newPoint = getTorusKnotPoint(data.t);
            
            positions[i * 3] = newPoint.x;
            positions[i * 3 + 1] = newPoint.y;
            positions[i * 3 + 2] = newPoint.z;
        }

        // Inform WebGL buffer that positions have changed
        particleGeometry.attributes.position.needsUpdate = true;
        
        // Slowly spin the entire particle system to align with Torus Knot rotation
        particleSystem.rotation.y += 0.005;
        particleSystem.rotation.x = Math.sin(Date.now() * 0.0004) * 0.12;
        particleSystem.rotation.z = Date.now() * 0.0001;
    }

    // 3. Mouse parallax tilt easing
    currentMouseX += (targetMouseX - currentMouseX) * 0.05;
    currentMouseY += (targetMouseY - currentMouseY) * 0.05;

    scene.rotation.y = currentMouseX * 0.3;
    scene.rotation.x = currentMouseY * 0.2;

    // 4. Cinematic Scroll-driven camera path movements
    // As the user scrolls, the camera zooms in closer and shifts downwards,
    // causing the background Torus Knot to dynamically grow and move off-center.
    camera.position.z = 7.5 - (scrollPercent * 3.2);
    camera.position.y = -(scrollPercent * 1.5);

    renderer.render(scene, camera);
}

// Start Three.js when page loaded
window.onload = init3D;
