#include <mosquitto.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/time.h>

#define BROKER "192.168.100.10"
#define PORT 1883
#define TOPIC "test"
#define CLIENT_ID "client1"

volatile int running = 1;

/* ================================
   Obtener tiempo actual en segundos
================================ */
double current_time()
{
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return tv.tv_sec + tv.tv_usec / 1000000.0;
}

/* ================================
   Callback cuando se recibe mensaje
================================ */
void on_message(struct mosquitto *mosq, void *userdata,
                const struct mosquitto_message *msg)
{
    printf("%s: %.*s\n", msg->topic, msg->payloadlen, (char *)msg->payload);
}

/* ================================
   Subscriber automático
================================ */
void run_subscriber()
{
    struct mosquitto *mosq;

    mosquitto_lib_init();

    mosq = mosquitto_new(CLIENT_ID, 1, NULL);

    if (!mosq)
    {
        printf("Error creating mosquitto instance\n");
        return;
    }

    mosquitto_message_callback_set(mosq, on_message);

    if (mosquitto_connect(mosq, BROKER, PORT, 60))
    {
        printf("Unable to connect\n");
        return;
    }

    mosquitto_subscribe(mosq, NULL, TOPIC, 1);

    while (1)
    {
        mosquitto_loop(mosq, -1, 1);
    }

    mosquitto_destroy(mosq);
    mosquitto_lib_cleanup();
}

/* ================================
   Publisher automático con controladores
================================ */
void run_publisher(int payload_size, int execution_time, int publish_frequency)
{
    struct mosquitto *mosq;
    char *payload;
    int message_count = 0;
    int total_messages = execution_time / publish_frequency;

    mosquitto_lib_init();

    mosq = mosquitto_new(CLIENT_ID, 1, NULL);

    if (!mosq)
    {
        printf("Error creating mosquitto instance\n");
        return;
    }

    if (mosquitto_connect(mosq, BROKER, PORT, 60))
    {
        printf("Unable to connect\n");
        return;
    }

    /* Crear payload automático */
    payload = malloc(payload_size + 1);
    memset(payload, 'A', payload_size);
    payload[payload_size] = '\0';

    /* Temporizador global */
    double program_start = current_time();

    while ((current_time() - program_start) < execution_time &&
           message_count < total_messages)
    {
        /* Temporizador de publicación */
        double publish_start = current_time();

        mosquitto_publish(
            mosq,
            NULL,
            TOPIC,
            payload_size,
            payload,
            1,
            0);

        message_count++;

        double elapsed = current_time() - publish_start;

        if (elapsed < publish_frequency)
        {
            usleep((publish_frequency - elapsed) * 1000000);
        }
    }

    printf("Total messages sent: %d\n", message_count);

    free(payload);

    mosquitto_disconnect(mosq);
    mosquitto_destroy(mosq);
    mosquitto_lib_cleanup();
}

/* ================================
   MAIN
================================ */
int main(int argc, char *argv[])
{
    if (argc < 2)
    {
        printf("Usage:\n");
        printf("Subscriber: %s sub\n", argv[0]);
        printf("Publisher: %s pub payload_size execution_time frequency\n", argv[0]);
        return 1;
    }

    if (strcmp(argv[1], "sub") == 0)
    {
        run_subscriber();
    }
    else if (strcmp(argv[1], "pub") == 0)
    {
        if (argc != 5)
        {
            printf("Usage: %s pub payload_size execution_time frequency\n", argv[0]);
            return 1;
        }

        int payload_size = atoi(argv[2]);
        int execution_time = atoi(argv[3]);
        int frequency = atoi(argv[4]);

        run_publisher(payload_size, execution_time, frequency);
    }

    return 0;
}