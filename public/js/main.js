// Tauri API imports
const { invoke } = window.__TAURI__.tauri;

// Application state
let currentUser = '';
let devices = [];
let transferProgress = [];

// ============================================================================
// Initialization
// ============================================================================

document.addEventListener('DOMContentLoaded', async () => {
    console.log('Desk Share Net initialized');

    // Set up event listeners
    setupEventListeners();

    // Load initial data
    await refreshDevices();

    // Start periodic refresh
    setInterval(refreshDevices, 5000);
});

// ============================================================================
// Event Listeners
// ============================================================================

function setupEventListeners() {
    // User name setup
    const userNameBtn = document.getElementById('set-username-btn');
    if (userNameBtn) {
        userNameBtn.addEventListener('click', setUserName);
    }

    // Device refresh
    const refreshBtn = document.getElementById('refresh-devices-btn');
    if (refreshBtn) {
        refreshBtn.addEventListener('click', refreshDevices);
    }

    // File transfer
    const fileTransferBtn = document.getElementById('start-transfer-btn');
    if (fileTransferBtn) {
        fileTransferBtn.addEventListener('click', startFileTransfer);
    }

    // Screen share
    const screenShareBtn = document.getElementById('start-screen-share-btn');
    if (screenShareBtn) {
        screenShareBtn.addEventListener('click', startScreenShare);
    }

    // Chat
    const sendChatBtn = document.getElementById('send-chat-btn');
    if (sendChatBtn) {
        sendChatBtn.addEventListener('click', sendChatMessage);
    }
}

// ============================================================================
// Tauri Command Handlers
// ============================================================================

async function setUserName() {
    const input = document.getElementById('username-input');
    if (!input || !input.value) return;

    try {
        const result = await invoke('set_user_name', { name: input.value });
        currentUser = input.value;
        console.log(result);
        showNotification('Success', `Username set to: ${input.value}`);
    } catch (error) {
        console.error('Failed to set username:', error);
        showNotification('Error', 'Failed to set username');
    }
}

async function refreshDevices() {
    try {
        devices = await invoke('get_devices');
        updateDeviceList(devices);

        // Also refresh the device list
        await invoke('refresh_devices');
    } catch (error) {
        console.error('Failed to refresh devices:', error);
    }
}

async function startFileTransfer() {
    const deviceIpInput = document.getElementById('device-ip-input');
    const filePathInput = document.getElementById('file-path-input');

    if (!deviceIpInput || !filePathInput) return;

    try {
        const result = await invoke('start_file_transfer', {
            deviceIp: deviceIpInput.value,
            filePath: filePathInput.value
        });
        console.log(result);
        showNotification('Success', 'File transfer started');
    } catch (error) {
        console.error('Failed to start file transfer:', error);
        showNotification('Error', 'Failed to start file transfer');
    }
}

async function startScreenShare() {
    const frameRateInput = document.getElementById('frame-rate-input');
    const frameRate = frameRateInput ? parseInt(frameRateInput.value) : 30;

    try {
        const sessionId = await invoke('start_screen_share', { frameRate });
        console.log('Screen share started:', sessionId);
        showNotification('Success', `Screen share started: ${sessionId}`);
    } catch (error) {
        console.error('Failed to start screen share:', error);
        showNotification('Error', 'Failed to start screen share');
    }
}

async function sendChatMessage() {
    const messageInput = document.getElementById('chat-message-input');
    const toInput = document.getElementById('chat-to-input');

    if (!messageInput || !messageInput.value) return;

    try {
        await invoke('send_chat_message', {
            message: messageInput.value,
            to: toInput && toInput.value ? toInput.value : null
        });
        messageInput.value = '';
        showNotification('Success', 'Message sent');
    } catch (error) {
        console.error('Failed to send message:', error);
        showNotification('Error', 'Failed to send message');
    }
}

// ============================================================================
// UI Update Functions
// ============================================================================

function updateDeviceList(devices) {
    const deviceList = document.getElementById('device-list');
    if (!deviceList) return;

    deviceList.innerHTML = '';

    if (devices.length === 0) {
        deviceList.innerHTML = '<p class="no-devices">No devices found</p>';
        return;
    }

    devices.forEach(device => {
        const deviceCard = createDeviceCard(device);
        deviceList.appendChild(deviceCard);
    });
}

function createDeviceCard(device) {
    const card = document.createElement('div');
    card.className = 'device-card';
    card.innerHTML = `
        <h3>${device.name}</h3>
        <p>IP: ${device.ip}:${device.port}</p>
        <p>Status: ${device.is_online ? 'Online' : 'Offline'}</p>
        <p>Last seen: ${device.last_seen}</p>
    `;
    return card;
}

function showNotification(title, message) {
    // Simple console notification for now
    // Can be enhanced with a proper notification UI
    console.log(`[${title}] ${message}`);

    // You can also use Tauri's notification API
    if (window.__TAURI__?.notification) {
        window.__TAURI__.notification.sendNotification({
            title,
            body: message
        });
    }
}
