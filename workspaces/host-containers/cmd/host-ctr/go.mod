module host-ctr

go 1.12

require (
	github.com/aws/aws-sdk-go v1.23.22
	github.com/awslabs/amazon-ecr-containerd-resolver v0.0.0-20190912214810-5bbc33959a5c
	github.com/containerd/cgroups v1.0.4 // indirect
	github.com/containerd/containerd v1.4.13
	github.com/containerd/go-runc v1.0.0 // indirect
	github.com/containerd/ttrpc v1.1.0 // indirect
	github.com/google/uuid v1.3.0 // indirect
	github.com/imdario/mergo v0.3.13 // indirect
	github.com/opencontainers/runc v1.0.0-rc8
	github.com/opencontainers/runtime-spec v1.0.2
	github.com/opencontainers/selinux v1.10.1 // indirect
	github.com/pkg/errors v0.9.1
	go.etcd.io/bbolt v1.3.6 // indirect
	google.golang.org/grpc v1.46.2 // indirect
	gotest.tools/v3 v3.2.0 // indirect
)

replace github.com/Sirupsen/logrus => github.com/sirupsen/logrus v1.4.2
