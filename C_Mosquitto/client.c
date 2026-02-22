#include <mosquitto.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/time.h>

#define BROKER "192.168.100.10"
#define PORT 1883
#define TOPIC "test"

volatile int running = 1;

/* ================================
   Tiempo actual
================================ */
double current_time()
{
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return tv.tv_sec + tv.tv_usec / 1000000.0;
}

/* ================================
   Callback subscriber
================================ */
void on_message(struct mosquitto *mosq, void *userdata,
                const struct mosquitto_message *msg)
{
    (void)mosq;
    (void)userdata;
}

/* ================================
   Subscriber
================================ */
void run_subscriber(int id)
{
    char client_id[50];
    sprintf(client_id, "sub_%d", id);

    mosquitto_lib_init();

    struct mosquitto *mosq =
        mosquitto_new(client_id, true, NULL);

    mosquitto_message_callback_set(mosq, on_message);

    mosquitto_connect(mosq, BROKER, PORT, 60);
    mosquitto_subscribe(mosq, NULL, TOPIC, 1);

    mosquitto_loop_forever(mosq, -1, 1);

    mosquitto_destroy(mosq);
    mosquitto_lib_cleanup();
}

/* ================================
   Publisher CONTROLADO
================================ */
void run_publisher(int id,
                   int payload_size,
                   int execution_time,
                   double publish_frequency)
{
    char client_id[50];
    sprintf(client_id, "pub_%d", id);

    mosquitto_lib_init();

    struct mosquitto *mosq =
        mosquitto_new(client_id, true, NULL);

    mosquitto_connect(mosq, BROKER, PORT, 60);

    char *payload = malloc(payload_size);
    memset(payload, 'A', payload_size);

    int message_count = 0;

    double program_start = current_time();
    double next_publish = program_start;

    while ((current_time() - program_start) < execution_time)
    {
        double now = current_time();

        /* Procesar red MQTT SIEMPRE */
        mosquitto_loop(mosq, 0, 1);

        if (now >= next_publish)
        {
            mosquitto_publish(
                mosq,
                NULL,
                TOPIC,
                payload_size,
                payload,
                1,
                0);

            message_count++;
            next_publish += publish_frequency;
        }

        usleep(1000);
    }

    printf("Publisher %d sent %d messages\n",
           id, message_count);

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
    if (argc < 3)
    {
        printf("Usage:\n");
        printf("sub id\n");
        printf("pub id payload exec_time freq\n");
        return 1;
    }

    if (!strcmp(argv[1], "sub"))
    {
        run_subscriber(atoi(argv[2]));
    }
    else if (!strcmp(argv[1], "pub"))
    {
        if (argc != 6)
        {
            printf("pub id payload exec_time freq\n");
            return 1;
        }

        run_publisher(
            atoi(argv[2]),
            atoi(argv[3]),
            atoi(argv[4]),
            atof(argv[5]));
    }

    return 0;
}