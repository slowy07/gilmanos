ARG SDK
FROM ${SDK} as sdk

# The experimental cache mount type doesn't expand arguments, so our choices are limited.
# We can either reuse the same cache for all builds, which triggers overlayfs errors if the
# builds run in parallel, or we can use a new cache for each build, which defeats the
# purpose. We work around the limitation by materializing a per-build stage that can be used
# as the source of the cache.
FROM scratch AS cache
ARG PACKAGE
ARG ARCH
ARG TOKEN
# We can't create directories via RUN in a scratch container, so take an existing one.
COPY --chown=1000:1000 --from=sdk /tmp /cache
# Ensure the ARG variables are used in the layer to prevent reuse by other builds.
COPY --chown=1000:1000 .dockerignore /cache/.${PACKAGE}.${ARCH}.${TOKEN}

# Some builds need to modify files in the source directory, for example Rust software using
# build.rs to generate code.  The source directory is mounted in using "--mount=source"
# which is owned by root, and we need to modify it as the builder user.  To get around this,
# we can use a "cache" mount, which we just won't share or reuse.  We mount a cache into the
# location we need to change, and in some cases, set up symlinks so that it looks like a
# normal part of the source tree.  (This is like a tmpfs mount, but cache mounts have more
# flexibility - you can specify a source to set them up beforehand, specify uid/gid, etc.)
# This cache is also variant-specific (in addition to package and arch, like the one above)
# for cases where we need to build differently per variant; the cache will be empty if you
# change BUILDSYS_VARIANT.
FROM scratch AS variantcache
ARG PACKAGE
ARG ARCH
ARG VARIANT
ARG TOKEN
# We can't create directories via RUN in a scratch container, so take an existing one.
COPY --chown=1000:1000 --from=sdk /tmp /variantcache
# Ensure the ARG variables are used in the layer to prevent reuse by other builds.
COPY --chown=1000:1000 .dockerignore /variantcache/.${PACKAGE}.${ARCH}.${VARIANT}.${TOKEN}


# Builds an RPM package from a spec file.
FROM sdk AS rpmbuild
ARG PACKAGE
ARG ARCH
ARG NOCACHE
ARG VARIANT
ARG REPO
ENV VARIANT=${VARIANT}
WORKDIR /home/builder

USER builder
ENV PACKAGE=${PACKAGE} ARCH=${ARCH}
COPY --chown=builder roles/${REPO}.root.json ./rpmbuild/BUILD/root.json
COPY ./macros/${ARCH} ./macros/shared ./macros/rust ./macros/cargo ./packages/${PACKAGE}/ .
RUN rpmdev-setuptree \
    && cat ${ARCH} shared rust caargo > .rpmmacros \
    && cat "%_cross_variant ${VARIANT}" >> .rpmmacros \
    && echo "%_cross_repo_root_json %{_builddir}/root.json" >> .rpmmacros \
    && rm ${ARCH} shared rust cargo \
    && mv *.spec rpmbuild/SPECS \
    && find . -maxdepth 1 -not -path '*/\.*' -type -f -exec mv {} rpmbuild/SOURCES/ \; \
    && echo {NONCACHE}

USER root
RUN --mount=target=/host \