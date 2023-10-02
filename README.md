# SSHKeySight
## Your Insightful Gateway to Secure Server Management

SSHKeySight is a tool that provides sysadmins with a clear view of SSH key access across multiple Virtual Machines (VMs). In a world where server security is critical, understanding who has access to your infrastructure is essential. With SSHKeySight, you can effortlessly see which keys grant access to which servers, streamlining the management process and bolstering security.

## Key Features:
<b>Transparent Overview</b>: Instantly see who has access to which VMs.

<b>User-friendly Interface</b>: No more sifting through convoluted logs or configurations.

<b>Enhanced Security</b>: Ensure only authorized personnel have access.

Dive in and experience a new standard in server management with SSHKeySight.


## Auto deploy to VMS

## Prerequisites:
You need a Unix-like system with a Bash shell.
Ensure you have SSH access to the servers listed in your CSV file.
wget or curl should be installed on your local system.
The remote servers should allow the operations your script is attempting to perform (file copying, command execution, etc.).
Instructions:
Prepare the CSV File:

1. Create a servers.csv file with lines in the format username,ip_address.
Example:
```
user,192.168.1.2
user,192.168.1.3
```

2. Prepare the Script:

Save your script in a file named deploy_ssh_key_client.sh (or another name of your choice).
Ensure the script has execute permissions. You can set this with the command:
```
chmod +x deploy_ssh_key_client.sh
```
3. Run the Script:

In the terminal, navigate to the directory containing your script and CSV file.
Run the script using the command:
```
./deploy_ssh_key_client.sh <VERSION> 
# Version example v1.0.2
```

The script will execute, and you should see debug messages as it processes each server in servers.csv.
### Notes:
Ensure your SSH key is loaded into the SSH agent if you're using key-based authentication. You might need to start the SSH agent and add your key using ssh-add /path/to/your/private/key if not already done.
If your script requires root permissions, you may need to run it with sudo ./deploy_ssh_key_client.sh.
Always be careful when running scripts that modify system configuration, especially on remote servers. Test on a small, non-production environment first to ensure everything works as expected.
Monitor the output of the script for any error messages or confirmations of success.

### Debugging:
If the script does not work as expected:

Check the debug messages printed by the script for clues.
Make sure there are no typos or errors in servers.csv.
Ensure you have network access to the servers listed in servers.csv.
Confirm that you have the necessary permissions to log into and modify each server in servers.csv.
By following these instructions, you should be able to successfully run your script on all servers listed in your servers.csv file.

## TODO
