#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <pthread.h>
#include <mosquitto.h>

#define BROKER_HOST "192.168.100.10"
#define BROKER_PORT 1883
#define KEEPALIVE   60

static struct mosquitto *mosq;
static int running = 1;

/* ---------- Callbacks ---------- */

static void on_connect(struct mosquitto *mosq, void *obj, int rc)
{
    if(rc == 0) {
        printf("[+] Connected to broker\n");
    } else {
        printf("[-] Connect failed: %d\n", rc);
    }
}

static void on_message(struct mosquitto *mosq, void *obj,
                       const struct mosquitto_message *msg)
{
    printf("\n[+] Message received\n");
    printf("    Topic: %s\n", msg->topic);
    printf("    Payload: %s\n\n", (char *)msg->payload);
}

static void on_disconnect(struct mosquitto *mosq, void *obj, int rc)
{
    printf("[+] Disconnected from broker (rc=%d)\n", rc);
}

/* ---------- MQTT loop thread ---------- */

void *mqtt_loop_thread(void *arg)
{
    mosquitto_loop_forever(mosq, -1, 1);
    return NULL;
}

/* ---------- MAIN ---------- */

int main(void)
{
    pthread_t mqtt_thread;
    char client_id[64];
    snprintf(client_id, sizeof(client_id), "client_%d", getpid());

    mosquitto_lib_init();

    mosq = mosquitto_new(client_id, true, NULL);
    if(!mosq) {
        fprintf(stderr, "[-] Error creating client\n");
        return 1;
    }

    /* Para cuando el broker NO es anónimo */
    // mosquitto_username_pw_set(mosq, "user", "password");

    mosquitto_connect_callback_set(mosq, on_connect);
    mosquitto_message_callback_set(mosq, on_message);
    mosquitto_disconnect_callback_set(mosq, on_disconnect);

    if(mosquitto_connect(mosq, BROKER_HOST, BROKER_PORT, KEEPALIVE) != MOSQ_ERR_SUCCESS) {
        fprintf(stderr, "[-] Unable to connect\n");
        return 1;
    }

    /* ---------- Start MQTT loop thread ---------- */
    pthread_create(&mqtt_thread, NULL, mqtt_loop_thread, NULL);

    /* ---------- Menu ---------- */
    while(running) {
        int choice;
        char topic[64];
        char message[256];

        printf("\n1. Publish\n");
        printf("2. Subscribe\n");
        printf("3. Exit\n");
        printf("> ");

        scanf("%d", &choice);
        getchar();  // limpiar buffer stdin

        if(choice == 1) {
            printf("Topic: ");
            fgets(topic, sizeof(topic), stdin);
            topic[strcspn(topic, "\n")] = 0;

            printf("Message: ");
            fgets(message, sizeof(message), stdin);
            message[strcspn(message, "\n")] = 0;

            mosquitto_publish(
                mosq,
                NULL,
                topic,
                strlen(message),
                message,
                1,
                false
            );
        }
        else if(choice == 2) {
            printf("Topic to subscribe: ");
            fgets(topic, sizeof(topic), stdin);
            topic[strcspn(topic, "\n")] = 0;

            mosquitto_subscribe(mosq, NULL, topic, 1);
            printf("[+] Subscribed to %s\n", topic);
        }
        else if(choice == 3) {
            running = 0;
        }
    }

    mosquitto_disconnect(mosq);
    mosquitto_destroy(mosq);
    mosquitto_lib_cleanup();

    return 0;
}
