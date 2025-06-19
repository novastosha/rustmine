# Variables
$mcVersion = "1.21.6"
$serverJar = "minecraft_server.$mcVersion.jar"
$serverUrl = "https://piston-data.mojang.com/v1/objects/HASH/server.jar"
$generatedDir = "generated"
$outputFiles = @(
    @{ src = "reports/blocks.json"; dst = "blocks.json" },
    @{ src = "reports/registries.json"; dst = "registries.json" },
    @{ src = "reports/biome_parameters/minecraft/*"; dst = "biomes" }
)

# Download server JAR if missing
if (-not (Test-Path $serverJar)) {
    Write-Host "Downloading Minecraft server $mcVersion..."
    Invoke-WebRequest -Uri $serverUrl -OutFile $serverJar
}

# Run the data generator
Write-Host "Running Minecraft data generator..."
java -DbundlerMainClass="net.minecraft.data.Main" -jar $serverJar --reports

# Ensure generated folder exists
if (-not (Test-Path $generatedDir)) {
    New-Item -ItemType Directory -Path $generatedDir | Out-Null
    
}

if (-not (Test-Path "$generatedDir\biomes")) {
    New-Item -ItemType Directory -Path "$generatedDir\biomes" | Out-Null
}

# Copy the generated files
foreach ($file in $outputFiles) {
    $src = Join-Path "./generated" $file.src
    $dst = Join-Path $generatedDir $file.dst
    if (Test-Path $src) {
        Copy-Item $src $dst -Force
        Write-Host "Copied $($file.dst) to $generatedDir"
    } else {
        Write-Warning "$($file.dst) not found at $src"
    }
}