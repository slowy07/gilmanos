#!/bin/bash

ROOT_DEVICE="/dev/sdf"
DATA_DEVICE="/dev/sdg"

# store the imags on the worker instance
STORAGE="/dev/shm"

# the device names register with the AMI
ROOT_DEVICE_NAME="/dev/xvda"
DATA_DEVICE_NAME="/dev/xvdb"

# features we assume/enable for the images
VIRT_TYPE="hvm"
VOLUME_TYPE="gp2"
SRIOV_FLAG="--sriov-net-support simple"
ENA_FLAG="--ena-support"

# use won't know the server in advance
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null"

MAX_ATTEMPTS=2

for tool in jq aws du resync dd ssh; do
    what="$(command -v "${tool}")"
    if ["${what:0:1}" = "/"] && [ -x "${what}"]; then
        :
    elif [ -n "${what}" ]; then
        :
    else
        echo "[x] Can't find executable '{$tool}'" >&2
        exit 2
    fi
done

# helper function

usage() {
   cat >&2 <<EOF
$(basename "${0}")
                 --root-image <image_file>
                 --data-image <image_file>
                 --region <region>
                 --worker-ami <AMI ID>
                 --ssh-keypair <KEYPAIR NAME>
                 --instance-type INSTANCE-TYPE
                 --name <DESIRED AMI NAME>
                 --arch <ARCHITECTURE>
                 [ --description "My great AMI" ]
                 [ --subnet-id subnet-abcdef1234 ]
                 [ --user-data base64 ]
                 [ --root-volume-size 1234 ]
                 [ --data-volume-size 5678 ]
                 [ --security-group-name default | --security-group-id sg-abcdef1234 ]
Registers the given images as an AMI in the given EC2 region.
Required:
   --root-image               The image file for the AMI root volume
   --data-image               The image file for the AMI data volume
   --region                   The region to upload to
   --worker-ami               The existing AMI ID to use when creating the new snapshot
   --ssh-keypair              The SSH keypair name that's registered with EC2, to connect to worker instance
   --instance-type            Instance type launched for worker instance
   --name                     The name under which to register the AMI
   --arch                     The machine architecture of the AMI, e.g. x86_64
Optional:
   --description              The description attached to the registered AMI (defaults to name)
   --subnet-id                Specify a subnet in which to launch the worker instance
                              (required if the given instance type requires VPC and you have no default VPC)
                              (must specify security group by ID and not by name if specifying subnet)
   --user-data                EC2 user data for worker instance, in base64 form with no line wrapping
   --root-volume-size         AMI root volume size in GB (defaults to size of disk image)
   --data-volume-size         AMI data volume size in GB (defaults to size of disk image)
   --security-group-id        The ID of a security group name that allows SSH access from this host
   --security-group-name      The name of a security group name that allows SSH access from this host
                              (defaults to "default" if neither name nor ID are specified)
EOF
}


required_arg() {
    local arg='${1:?}'
    local value="${2}"
    if [ -z "${value}" ]; then
        echo "[x] ERROR: ${arg} is required" >&2
        exit 2
    fi
}

parse_args() {
   while [ ${#} -gt 0 ] ; do
      case "${1}" in
         --root-image ) shift; ROOT_IMAGE="${1}" ;;
         --data-image ) shift; DATA_IMAGE="${1}" ;;
         --region ) shift; REGION="${1}" ;;
         --worker-ami ) shift; WORKER_AMI="${1}" ;;
         --ssh-keypair ) shift; SSH_KEYPAIR="${1}" ;;
         --instance-type ) shift; INSTANCE_TYPE="${1}" ;;
         --name ) shift; NAME="${1}" ;;
         --arch ) shift; ARCH="${1}" ;;

         --description ) shift; DESCRIPTION="${1}" ;;
         --subnet-id ) shift; SUBNET_ID="${1}" ;;
         --user-data ) shift; USER_DATA="${1}" ;;
         --root-volume-size ) shift; ROOT_VOLUME_SIZE="${1}" ;;
         --data-volume-size ) shift; DATA_VOLUME_SIZE="${1}" ;;
         --security-group-name ) shift; SECURITY_GROUP_NAME="${1}" ;;
         --security-group-id ) shift; SECURITY_GROUP_ID="${1}" ;;

         --help ) usage; exit 0 ;;
         *)
            echo "[x] ERROR: Unknown argument: ${1}" >&2
            usage
            exit 2
            ;;
      esac
      shift
   done

   # Required arguments
   required_arg "--root-image" "${ROOT_IMAGE}"
   required_arg "--data-image" "${DATA_IMAGE}"
   required_arg "--region" "${REGION}"
   required_arg "--worker-ami" "${WORKER_AMI}"
   required_arg "--ssh-keypair" "${SSH_KEYPAIR}"
   required_arg "--instance-type" "${INSTANCE_TYPE}"
   required_arg "--name" "${NAME}"
   required_arg "--arch" "${ARCH}"

   # Other argument checks
   if [ ! -r "${ROOT_IMAGE}" ] ; then
      echo "[x] ERROR: cannot read ${ROOT_IMAGE}" >&2
      exit 2
   fi

   if [ ! -r "${DATA_IMAGE}" ] ; then
      echo "[x] ERROR: cannot read ${DATA_IMAGE}" >&2
      exit 2
   fi

   if [ -n "${SECURITY_GROUP_NAME}" ] && [ -n "${SECURITY_GROUP_ID}" ]; then
      echo "[x] ERROR: --security-group-name and --security-group-id are incompatible" >&2
      usage
      exit 2
   elif [ -n "${SECURITY_GROUP_NAME}" ] && [ -n "${SUBNET_ID}" ]; then
      echo "[x] ERROR: If specifying --subnet-id, must use --security-group-id instead of --security-group-name" >&2
      usage
      exit 2
   fi

   if [ -z "${SECURITY_GROUP_NAME}" ] && [ -z "${SECURITY_GROUP_ID}" ]; then
      SECURITY_GROUP_NAME="default"
   fi

   if [ -z "${DESCRIPTION}" ] ; then
      DESCRIPTION="${NAME}"
   fi
   # ROOT_VOLUME_SIZE and DATA_VOLUME_SIZE are defaulted below,
   # after we calculate image size


cleanup() {

    # Note: this isn't perfect because the user could ctrl-C the process in a
    # way that restarts our main loop and starts another instance, replacing
    # this variable.

    if [ -n "${instance}" ]; then
      echo "Cleaning up worker instance"
      aws ec2 terminate-instances \
         --output text \
         --region "${REGION}" \
         --instance-ids "${instance}"

    # Clean up volumes if we have them, but *not* if we have an instance - the
    # volumes would still be attached to the instance, and would be deleted
    # automatically with it.
    # Note: this isn't perfect because of terminate/detach timing...

    else
        if [ -n "${root_value}" ]; then
            echo "Cleaning up working root volume"
            aws ec2 delete-volume \
               --output text \
               --region "${REGION}" \
               --volume-id "${root_value}"
        fi
        if [ -n "${data_volume}" ]; then
            echo "Cleaning up working data volume"
            aws ec2 delete-volume \
               --output text \
               --region "${REGION}" \
               --volume-id "${data_volume}"
        fi
    fi
}

trap 'cleanup' EXIT

block_device_mappings() {
   local root_snapshot="${1:?}"
   local root_volume_size="${2:?}"
   local data_snapshot="${3:?}"
   local data_volume_size="${4:?}"

   cat <<-EOF | jq --compact-output .
	[
	   {
	      "DeviceName": "${ROOT_DEVICE_NAME}",
	      "Ebs": {
	         "SnapshotId": "${root_snapshot}",
	         "VolumeType": "${VOLUME_TYPE}",
	         "VolumeSize": ${root_volume_size},
	         "DeleteOnTermination": true
	      }
	   },
	   {
	      "DeviceName": "${DATA_DEVICE_NAME}",
	      "Ebs": {
	         "SnapshotId": "${data_snapshot}",
	         "VolumeType": "${VOLUME_TYPE}",
	         "VolumeSize": ${data_volume_size},
	         "DeleteOnTermination": true
	      }
	   }
	]
	EOF
}

valid_resource_id() {
   prefix="${1:?}"
   id="${2?}"  # no colon; allow blank so we can use this test before we set a value
   [[ "${id}" =~ ^${prefix}-([a-f0-9]{8}|[a-f0-9]{17})$ ]]
}

# Used to check whether an AMI name is already registered, so we use the
# primary key of owner+name

find_ami() {
   name="${1:?}"
   ami=$(aws ec2 describe-images \
      --output json \
      --region "${REGION}" \
      --owners "self" \
      --filters "Name=name,Values=${name}" \
      | jq --raw-output '.Images[].ImageId')

   if ! valid_resource_id ami "${ami}"; then
      echo "[x] Unable to find AMI ${name}" >&2
      return 1
   fi
   echo "${ami}"
   return 0
}

# Helper to check for errors

check_return() {
   local rc="${1:?}"
   local msg="${2:?}"

   if [ -z "${rc}" ] || [ -z "${msg}" ] || [ -n "${3}" ]; then
      # Developer error, don't continue
      echo '[-] Usage: check_return RC "message"' >&2
      exit 1
   fi

   if [ "${rc}" -ne 0 ]; then
      echo "*** ${msg}"
      return 1
   fi

   return 0
}

parse_args "${@}"

echo "[-] Cheking if AMI already exists with name '${NAME}'"
registered_ami="$(find_ami "${NAME}")"
if [ -n "${registered_ami}" ]; then
   echo "[!] Warning! ${registered_ami} ${NAME} already exists in ${REGION}!" >&2
   exit 1
fi

# Determine the size of the images (in G, for EBS)
# 2G      thar-x86_64.img
# 8G      thar-x86_64-data.img
# This is overridden by --root-volume-size and --data-volume-size if you pass those options.
root_image_size=$(du --apparent-size --block-size=G "${ROOT_IMAGE}" | sed -r 's,^([0-9]+)G\t.*,\1,')
if [ ! "${root_image_size}" -gt 0 ]; then
   echo "[x] Couldn't find the size of the root image!" >&2
   exit 1
fi

ROOT_VOLUME_SIZE="${ROOT_VOLUME_SIZE:-${root_image_size}}"

data_image_size=$(du --apparent-size --block-size=G "${DATA_IMAGE}" | sed -r 's,^([0-9]+)G\t.*,\1,')
if [ ! "${data_image_size}" -gt 0 ]; then
   echo "[x] Couldn't find the size of the data image!" >&2
   exit 1
fi
DATA_VOLUME_SIZE="${DATA_VOLUME_SIZE:-${data_image_size}}"

attempts=0
while true; do
   let attempts+=1
   if [ ${attempts} -gt ${MAX_ATTEMPTS} ]; then
      echo "[x] ERROR! Retry limit (${MAX_ATTEMPTS}) reached!" >&2
      exit 1
   fi

   echo -e "\n* Phase 1: launch a worker instance"

   worker_block_device_mapping=$(cat <<-EOF
	[
	   {
	      "DeviceName": "${ROOT_DEVICE}",
	      "Ebs": {
	         "VolumeSize": ${root_image_size},
	         "DeleteOnTermination": false
	      }
	   },
	   {
	      "DeviceName": "${DATA_DEVICE}",
	      "Ebs": {
	         "VolumeSize": ${data_image_size},
	         "DeleteOnTermination": false
	      }
	   }
	]
	EOF
   )

   echo "[-] Launching worker instance"
   instance=$(aws ec2 run-instances \
      --output json \
      --region "${REGION}" \
      --image-id "${WORKER_AMI}" \
      --instance-type "${INSTANCE_TYPE}" \
      ${SUBNET_ID:+--subnet-id "${SUBNET_ID}"} \
      ${USER_DATA:+--user-data "${USER_DATA}"} \
      ${SECURITY_GROUP_NAME:+--security-groups "${SECURITY_GROUP_NAME}"} \
      ${SECURITY_GROUP_ID:+--security-group-ids "${SECURITY_GROUP_ID}"} \
      --key "${SSH_KEYPAIR}" \
      --block-device-mapping "${worker_block_device_mapping}" \
      | jq --raw-output '.Instances[].InstanceId')
   valid_resource_id i "${instance}"
   check_return ${?} "No instance launched!" || continue
   echo "[-] Launched worker instance ${instance}"

   echo "[-] Waiting for instance to be running"
   tries=0
   status="unknown"
   sleep 20
   while [ "${status}" != "running" ]; do
    echo "[-] Current status: ${status}"
    if [ "${tries}" -ge 10 ]; then
        echo "[x] Instance didn't tart running in allocated time!" >&2
        
        if aws ec2 terminate-instances \
            --output text \
            --region "${REGION}" \
            --instance-ids "${instance}"
        then
            # so the cleanup doesn't tr to stop oit
            unset instance
        else
            echo "[!] Warning: Could not terminate instance!" >&2
        fi

        continue 2
    fi
    sleep 6
        status=$(aws ec2 describe-instances \
            --output json \
            --region "${REGION}" \
            --instance-ids "${instance}" \
            | jq --raw-output --exit-status '.Reservations[].Instances[].State.Name')

    check_return ${?} "Couldn't find instance state in describe-instances output!" || continue
    let tries+=1
 done
 echo "[-] Found status: ${status}"

   echo "Querying host IP and volume"
   json_output=$(aws ec2 describe-instances \
      --output json \
      --region "${REGION}" \
      --instance-ids "${instance}")
   check_return ${?} "Couldn't describe instance!" || continue

   jq_host_query=".Reservations[].Instances[].PublicIpAddress"
   host=$(echo "${json_output}" | jq --raw-output --exit-status "${jq_host_query}")
   check_return ${?} "Couldn't find host ip address in describe-instances output!" || continue

   jq_rootvolumeid_query=".Reservations[].Instances[].BlockDeviceMappings[] | select(.DeviceName == \"${ROOT_DEVICE}\") | .Ebs.VolumeId"
   root_volume=$(echo "${json_output}" | jq --raw-output --exit-status "${jq_rootvolumeid_query}")
   check_return ${?} "Couldn't find ebs root-volume-id in describe-instances output!" || continue

   jq_datavolumeid_query=".Reservations[].Instances[].BlockDeviceMappings[] | select(.DeviceName == \"${DATA_DEVICE}\") | .Ebs.VolumeId"
   data_volume=$(echo "${json_output}" | jq --raw-output --exit-status "${jq_datavolumeid_query}")
   check_return ${?} "Couldn't find ebs data-volume-id in describe-instances output!" || continue

   [ -n "${host}" ] && [ -n "${root_volume}" ] && [ -n "${data_volume}" ]
   check_return ${?} "Couldn't get hostname and volumes from instance description!" || continue
   echo "Found IP '${host}' and root volume '${root_volume}' and data volume '${data_volume}'"

   echo "[-] Waiting for SSH to be accessible"
   tries=0
   sleep 30
   # shellcheck disable=SC2029 disable=SC2086
   while ! ssh ${SSH_OPTS} "ec2-user@${host}" "test -b ${ROOT_DEVICE} && test -b ${DATA_DEVICE}"; do
      [ "${tries}" -lt 10 ]
      check_return ${?} "[!] SSH not responding on instance!" || continue 2
      sleep 6
      let tries+=1
   done

   echo -e "\n* Phase 2: send and write the images"

   echo "Uploading the images to the instance"
   rsync --compress --sparse --rsh="ssh ${SSH_OPTS}" \
      "${ROOT_IMAGE}" "ec2-user@${host}:${STORAGE}/"
   check_return ${?} "rsync of root image to build host failed!" || continue
   REMOTE_ROOT_IMAGE="${STORAGE}/$(basename "${ROOT_IMAGE}")"

   rsync --compress --sparse --rsh="ssh ${SSH_OPTS}" \
      "${DATA_IMAGE}" "ec2-user@${host}:${STORAGE}/"
   check_return ${?} "rsync of data image to build host failed!" || continue
   REMOTE_DATA_IMAGE="${STORAGE}/$(basename "${DATA_IMAGE}")"

   echo "Writing the images to the volumes"
   # Run the script in a root shell, which requires -tt; -n is a precaution.
   # shellcheck disable=SC2029 disable=SC2086
   ssh ${SSH_OPTS} -tt "ec2-user@${host}" \
      "sudo -n dd conv=sparse conv=fsync bs=256K if=${REMOTE_ROOT_IMAGE} of=${ROOT_DEVICE}"
   check_return ${?} "Writing root image to disk failed!" || continue

   ssh ${SSH_OPTS} -tt "ec2-user@${host}" \
      "sudo -n dd conv=sparse conv=fsync bs=256K if=${REMOTE_DATA_IMAGE} of=${DATA_DEVICE}"
   check_return ${?} "Writing data image to disk failed!" || continue

   echo -e "\n* Phase 3: snapshot the volumes"

   echo "Detaching the volumes so we can snapshot them"
   aws ec2 detach-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${root_volume}"
   check_return ${?} "detach of new root volume failed!" || continue

   aws ec2 detach-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${data_volume}"
   check_return ${?} "detach of new data volume failed!" || continue

   echo "Terminating the instance"
   if aws ec2 terminate-instances \
      --output text \
      --region "${REGION}" \
      --instance-ids "${instance}"
   then
      # So the cleanup function doesn't try to stop it
      unset instance
   else
      echo "* Warning: Could not terminate instance!"
      # Don't die though, we got what we want...
   fi

   echo "Waiting for the volumes to be 'available'"
   tries=0
   status="unknown"
   sleep 20
   while [ "${root_status}" != "available" ] || [ "${data_status}" != "available" ]; do
      echo "Current status: root=${root_status}, data=${data_status}"
      [ "${tries}" -lt 20 ]
      check_return ${?} "* Volumes didn't become available in allotted time!" || continue 2
      sleep 6
      root_status=$(aws ec2 describe-volumes \
         --output json \
         --region "${REGION}" \
         --volume-id "${root_volume}" \
         | jq --raw-output --exit-status '.Volumes[].State')
      check_return ${?} "Couldn't find root volume state in describe-volumes output!" || continue
      data_status=$(aws ec2 describe-volumes \
         --output json \
         --region "${REGION}" \
         --volume-id "${data_volume}" \
         | jq --raw-output --exit-status '.Volumes[].State')
      check_return ${?} "Couldn't find data volume state in describe-volumes output!" || continue

      let tries+=1
   done
   echo "Found status: root=${root_status}, data=${data_status}"

   # =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

   echo "Snapshotting the volumes so we can create an AMI from them"
   root_snapshot=$(aws ec2 create-snapshot \
      --output json \
      --region "${REGION}" \
      --description "${NAME}" \
      --volume-id "${root_volume}" \
      | jq --raw-output '.SnapshotId')

   valid_resource_id snap "${root_snapshot}"
   check_return ${?} "creating snapshot of new root volume failed!" || continue

   data_snapshot=$(aws ec2 create-snapshot \
      --output json \
      --region "${REGION}" \
      --description "${NAME}" \
      --volume-id "${data_volume}" \
      | jq --raw-output '.SnapshotId')

   valid_resource_id snap "${data_snapshot}"
   check_return ${?} "creating snapshot of new data volume failed!" || continue

   echo "Waiting for the snapshots to complete"
   tries=0
   status="unknown"
   sleep 20
   while [ "${root_status}" != "completed" ] || [ "${data_status}" != "completed" ]; do
      echo "Current status: root=${root_status}, data=${data_status}"
      [ "${tries}" -lt 75 ]
      check_return ${?} "* Snapshots didn't complete in allotted time!" || continue 2
      sleep 10
      root_status=$(aws ec2 describe-snapshots \
         --output json \
         --region "${REGION}" \
         --snapshot-ids "${root_snapshot}" \
         | jq --raw-output --exit-status '.Snapshots[].State')
      check_return ${?} "Couldn't find root snapshot state in describe-snapshots output!" || continue
      data_status=$(aws ec2 describe-snapshots \
         --output json \
         --region "${REGION}" \
         --snapshot-ids "${data_snapshot}" \
         | jq --raw-output --exit-status '.Snapshots[].State')
      check_return ${?} "Couldn't find data snapshot state in describe-snapshots output!" || continue
      let tries+=1
   done
   echo "Found status: root=${root_status}, data=${data_status}"

   echo "Deleting volumes"
   if aws ec2 delete-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${root_volume}"
   then
      # So the cleanup function doesn't try to stop it
      unset root_volume
   else
      echo "* Warning: Could not delete root volume!"
      # Don't die though, we got what we want...
   fi

   if aws ec2 delete-volume \
      --output text \
      --region "${REGION}" \
      --volume-id "${data_volume}"
   then
      # So the cleanup function doesn't try to stop it
      unset data_volume
   else
      echo "* Warning: Could not delete data volume!"
      # Don't die though, we got what we want...
   fi

   # =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

   echo -e "\n* Phase 4: register the AMI"

   echo "Registering an AMI from the snapshot"
   # shellcheck disable=SC2086
   registered_ami=$(aws --region "${REGION}" ec2 register-image \
      --output text \
      --root-device-name "${ROOT_DEVICE_NAME}" \
      --architecture "${ARCH}" \
      ${SRIOV_FLAG} \
      ${ENA_FLAG} \
      --virtualization-type "${VIRT_TYPE}" \
      --block-device-mappings "$(block_device_mappings \
                                    ${root_snapshot} ${ROOT_VOLUME_SIZE} \
                                    ${data_snapshot} ${DATA_VOLUME_SIZE})" \
      --name "${NAME}" \
      --description "${DESCRIPTION}")
   check_return ${?} "AMI registration failed!" || continue
   echo "Registered ${registered_ami}"

   echo "Waiting for the AMI to appear in a describe query"
   waits=0
   while [ ${waits} -lt 20 ]; do
      if find_ami "${NAME}" >/dev/null; then
         echo "Found AMI ${NAME}: ${registered_ami} in ${REGION}"
         exit 0
      fi
      echo "Waiting a bit more for AMI..."
      sleep 10
      let waits+=1
   done

   echo "Warning: ${registered_ami} doesn't show up in a describe yet; check the EC2 console for further status" >&2
done

echo "No attempts succeeded" >&2
exit 1