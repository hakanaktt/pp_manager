# Test script to validate the Tauri build process
# Run this script to test if the application can be built successfully

Write-Host "Testing Process Priority Manager (Tauri Edition) Build Process" -ForegroundColor Green
Write-Host "================================================================" -ForegroundColor Green

# Check prerequisites
Write-Host "`nChecking prerequisites..." -ForegroundColor Yellow

# Check Node.js
try {
    $nodeVersion = node --version
    Write-Host "✓ Node.js found: $nodeVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Node.js not found. Please install Node.js 18+ from https://nodejs.org/" -ForegroundColor Red
    exit 1
}

# Check npm
try {
    $npmVersion = npm --version
    Write-Host "✓ npm found: $npmVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ npm not found. Please install npm" -ForegroundColor Red
    exit 1
}

# Check Rust
try {
    $rustVersion = rustc --version
    Write-Host "✓ Rust found: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust not found. Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check Cargo
try {
    $cargoVersion = cargo --version
    Write-Host "✓ Cargo found: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "✗ Cargo not found. Please install Rust toolchain" -ForegroundColor Red
    exit 1
}

# Install dependencies
Write-Host "`nInstalling dependencies..." -ForegroundColor Yellow
try {
    npm install
    Write-Host "✓ Dependencies installed successfully" -ForegroundColor Green
} catch {
    Write-Host "✗ Failed to install dependencies" -ForegroundColor Red
    exit 1
}

# Test TypeScript compilation
Write-Host "`nTesting TypeScript compilation..." -ForegroundColor Yellow
try {
    npm run vue-tsc
    Write-Host "✓ TypeScript compilation successful" -ForegroundColor Green
} catch {
    Write-Host "✗ TypeScript compilation failed" -ForegroundColor Red
    exit 1
}

# Test frontend build
Write-Host "`nTesting frontend build..." -ForegroundColor Yellow
try {
    npm run build
    Write-Host "✓ Frontend build successful" -ForegroundColor Green
} catch {
    Write-Host "✗ Frontend build failed" -ForegroundColor Red
    exit 1
}

# Test Rust compilation
Write-Host "`nTesting Rust backend compilation..." -ForegroundColor Yellow
try {
    Set-Location src-tauri
    cargo check
    Set-Location ..
    Write-Host "✓ Rust backend compilation successful" -ForegroundColor Green
} catch {
    Write-Host "✗ Rust backend compilation failed" -ForegroundColor Red
    Set-Location ..
    exit 1
}

Write-Host "`n================================================================" -ForegroundColor Green
Write-Host "All tests passed! The application should build successfully." -ForegroundColor Green
Write-Host "`nTo build the application, run:" -ForegroundColor Cyan
Write-Host "  npm run tauri build" -ForegroundColor White
Write-Host "`nTo run in development mode:" -ForegroundColor Cyan
Write-Host "  npm run tauri dev" -ForegroundColor White
Write-Host "================================================================" -ForegroundColor Green
