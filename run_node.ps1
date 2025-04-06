# ELXR Node Launch Script
param (
    [string]$WsPort = "9944",
    [string]$RpcPort = "9933",
    [string]$RelayChainArgs = "",
    [string]$QuantumArgs = ""
)

Write-Host "Starting ELXR Node..." -ForegroundColor Green

# Set current directory
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

# Define node path
$nodePath = "$scriptDir\target\release\elxr-node.exe"

# Launch node
if (Test-Path $nodePath) {
    Write-Host "Using existing executable: $nodePath" -ForegroundColor Cyan
    & $nodePath --chain=elxr-local --collator --ws-port $WsPort --rpc-port $RpcPort --parachain-id 2001 $RelayChainArgs $QuantumArgs
} else {
    Write-Host "ELXR executable not found at: $nodePath" -ForegroundColor Red
    Write-Host "Please build the node first with: cargo build --release" -ForegroundColor Yellow
}
