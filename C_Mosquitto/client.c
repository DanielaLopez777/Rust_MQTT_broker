#include <mosquitto.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/time.h>

#define BROKER "192.168.100.10"
#define PORT 1883
#define TOPIC "test"

double current_time()
{
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return tv.tv_sec + tv.tv_usec / 1000000.0;
}

/* ================================
   MESSAGE CALLBACK
================================ */
void on_message(struct mosquitto *mosq, void *userdata,
                const struct mosquitto_message *msg)
{
    (void)mosq;
    (void)userdata;
}

/* ================================
   SUBSCRIBER
================================ */
void run_subscriber()
{
    char client_id[50];
    sprintf(client_id, "sub_%d", getpid());

    mosquitto_lib_init();

    struct mosquitto *mosq =
        mosquitto_new(client_id, 1, NULL);

    mosquitto_message_callback_set(mosq, on_message);

    if (mosquitto_connect(mosq, BROKER, PORT, 60))
    {
        printf("Connection error\n");
        return;
    }

    mosquitto_subscribe(mosq, NULL, TOPIC, 1);

    mosquitto_loop_forever(mosq, -1, 1);

    mosquitto_destroy(mosq);
    mosquitto_lib_cleanup();
}

/* ================================
   PUBLISHER
================================ */
void run_publisher(int payload_size,
                   int execution_time,
                   double publish_frequency)
{
    char client_id[50];
    sprintf(client_id, "pub_%d", getpid());

    mosquitto_lib_init();

    struct mosquitto *mosq =
        mosquitto_new(client_id, 1, NULL);

    if (mosquitto_connect(mosq, BROKER, PORT, 60))
    {
        printf("Connection error\n");
        return;
    }

    char *payload = malloc(payload_size);
    memset(payload, 'A', payload_size);

    int message_count = 0;
    double start = current_time();

    while ((current_time() - start) < execution_time)
    {
        double t0 = current_time();

        mosquitto_publish(
            mosq,
            NULL,
            TOPIC,
            payload_size,
            payload,
            1,
            0);

        message_count++;

        double elapsed = current_time() - t0;

        if (elapsed < publish_frequency)
            usleep((publish_frequency - elapsed) * 1000000);
    }

    printf("PID %d sent %d messages\n", getpid(), message_count);

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
        printf("%s sub\n", argv[0]);
        printf("%s pub payload exec_time frequency\n", argv[0]);
        return 1;
    }

    if (!strcmp(argv[1], "sub"))
        run_subscriber();

    else if (!strcmp(argv[1], "pub"))
    {
        int payload = atoi(argv[2]);
        int exec_time = atoi(argv[3]);
        double freq = atof(argv[4]);

        run_publisher(payload, exec_time, freq);
    }

    return 0;
}