#!/bin/bash
# Exit on error
set -e

echo "=== Packaging AirBoard Unsigned IPA ==="

# 1. Ensure we are in the project root directory
if [ ! -d "build" ]; then
  echo "Error: 'build' directory not found. Please run this script from the project root."
  exit 1
fi

# 2. Create a temporary folder
echo "Creating temporary Payload directory..."
mkdir -p Payload

# 3. Locate the app bundle from the xcarchive folder or build folder
APP_PATH=""
if [ -d "build/ios/archive/Runner.xcarchive/Products/Applications/Runner.app" ]; then
  APP_PATH="build/ios/archive/Runner.xcarchive/Products/Applications/Runner.app"
elif [ -d "build/ios/iphoneos/Runner.app" ]; then
  APP_PATH="build/ios/iphoneos/Runner.app"
else
  echo "Error: Runner.app not found. Please make sure to run 'flutter build ipa --release --no-codesign' first."
  rm -rf Payload
  exit 1
fi

echo "Copying app bundle from: $APP_PATH"
cp -r "$APP_PATH" Payload/

# 4. Zip the folder and name it as an .ipa, saving it into releases/
echo "Packaging into releases/AirBoard-ipadosios.ipa..."
mkdir -p releases
zip -vr releases/AirBoard-ipadosios.ipa Payload/

# 5. Delete the temporary folder
echo "Cleaning up temporary folder..."
rm -rf Payload

echo "=== Packaging Completed Successfully! ==="
