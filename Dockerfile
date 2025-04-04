# Этап сборки для aarch64
FROM rust:1.82.0 as builder
ENV DEBIAN_FRONTEND=noninteractive
LABEL name="uksivtbotrust" version="0.1.3"

# Устанавливаем рабочую директорию в контейнере сборки
WORKDIR /usr/src/app

# Устанавливаем необходимые пакеты для кросс-компиляции
RUN apt-get update && apt-get install -y --no-install-recommends \
    software-properties-common \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libssl-dev \
    libleptonica-dev \
    pkg-config \
    ca-certificates  \
    llvm \
    clang \
    binutils \
    lld \
    libsqlite3-0 \
    openssl \
    binutils-aarch64-linux-gnu \
    && rm -rf /var/lib/apt/lists/*

# Копируем файл Cargo.toml и Cargo.lock для кэширования зависимостей
COPY Cargo.toml Cargo.lock ./

# Копируем исходный код
COPY src ./src

# Собираем проект без специфики архитектуры
#RUN cargo build --release --target aarch64-unknown-linux-gnu
# Кросс-компиляция для целевой архитектуры - закомментирована
RUN cargo build --release
#RUN cargo build --release
# Просматриваем содержимое директории релизов
#RUN ls -la /usr/src/app/target/aarch64-unknown-linux-gnu/release/
# Просматриваем содержимое target для aarch64 - закомментировано
RUN ls -la /usr/src/app/target/release/
#RUN ls -la /usr/src/app/target/release/
# Этап исполнения для aarch64
FROM ubuntu:22.04 as runner
ENV DEBIAN_FRONTEND=noninteractive

# Устанавливаем рабочую директорию в контейнере исполнения
WORKDIR /usr/src/app

# Устанавливаем часовой пояс
RUN ln -snf /usr/share/zoneinfo/Asia/Yekaterinburg /etc/localtime && echo "Asia/Yekaterinburg" > /etc/timezone

# Устанавливаем необходимые пакеты для запуска
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    libleptonica-dev \
    libsqlite3-0 \
    ca-certificates  \
    openssl \
    && rm -rf /var/lib/apt/lists/*
ENV LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH

# Копируем исполняемый файл из этапа сборки
COPY --from=builder /usr/src/app/target/release/UKSIVTbot ./uksivtbot
# Копируем файл с целевой архитектуры - закомментировано
# COPY --from=builder /usr/src/app/target/aarch64-unknown-linux-gnu/release/UKSIVTbot ./uksivtbot
#COPY --from=builder /usr/src/app/target/release/UKSIVTbot ./uksivtbot
# Переименовываем исполняемый файл
RUN mv uksivtbot Start

# Запускаем приложение
CMD ["./Start"]