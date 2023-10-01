#!/bin/bash

# Set the version as a variable
VERSION="v1.0.2"
URL="https://github.com/isankadn/SSHKeySight/releases/download/$VERSION/ssh_key_client"

# Download the file locally using either wget or curl
if command -v wget > /dev/null; then
    wget -O ./ssh_key_client $URL
elif command -v curl > /dev/null; then
    curl -L -o ./ssh_key_client $URL
else
    echo "Neither wget nor curl is available on this system. Please install either to proceed."
    exit 1
fi

if [ ! -f ./ssh_key_client ]; then
    echo "Failed to download ssh_key_client from $URL"
    exit 1
fi

IFS=$'\n' # Set Internal Field Separator to newline for the for loop
for line in $(cat servers.csv); do
    IFS=, read -r username ip <<< "$line" # Set Internal Field Separator to comma for read command
    server="$username@$ip"

    if [ -z "$server" ]; then
        continue
    fi

    # username=$(echo $server | cut -d'@' -f1)
    # ip=$(echo $server | cut -d'@' -f2)

    echo "DEBUG: Deploying to $ip as $username with $URL"

    # Get the current directory on the target server
    TARGET_DIR=$(ssh -o "StrictHostKeyChecking=no" "$server" 'pwd')
    if [ $? -ne 0 ]; then
        echo "DEBUG: Failed to retrieve the target directory for $server, continuing with next server..."
        continue  # Skip to the next iteration
    fi

    # Copy the file to the target server
    scp -o "StrictHostKeyChecking=no" ./ssh_key_client "$server:$TARGET_DIR/"
    if [ $? -ne 0 ]; then
        echo "DEBUG: Copy failed for $server, continuing with next server..."
        continue  # Skip to the next iteration
    fi

    # Set the necessary permissions
    ssh -o "StrictHostKeyChecking=no" "$server" "chmod +x $TARGET_DIR/ssh_key_client"

    echo "DEBUG: Setting up crontab for $ip"
    # Setup the crontab on the target server
    ssh -o "StrictHostKeyChecking=no" "$server" "
        crontab -l | grep -v 'ssh_key_client' | crontab -
        (crontab -l 2>/dev/null; echo '0 */4 * * * sudo $TARGET_DIR/ssh_key_client >> $TARGET_DIR/ssh_key_client.log 2>&1') | crontab -
    "
done 

# Remove the locally downloaded file
rm ./ssh_key_client

echo "Deployment complete!"
# Remove the locally downloaded file



