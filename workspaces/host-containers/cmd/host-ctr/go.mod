module host-ctr

go 1.12

require (
	github.com/aws/aws-sdk-go v1.23.22
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20190912214810-5bbc33959a5c
	github.com/containerd/cgroups v1.0.4 // indirect
	github.com/containerd/containerd v1.5.13
	github.com/google/uuid v1.3.0 // indirect
	github.com/opencontainers/runc v1.1.2
	github.com/opencontainers/runtime-spec v1.0.3-0.20210326190908-1c3f411f0417
	github.com/opencontainers/selinux v1.10.1 // indirect
	github.com/pkg/errors v0.9.1
	google.golang.org/grpc v1.46.2 // indirect
	gopkg.in/yaml.v3 v3.0.0 // indirect
	gotest.tools/v3 v3.2.0 // indirect
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.4.2
