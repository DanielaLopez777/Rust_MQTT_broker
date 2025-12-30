#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <mosquitto.h>

#define BROKER_HOST "192.168.100.10"
#define BROKER_PORT 1883
#define KEEPALIVE   60

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

int main(void)
{
    struct mosquitto *mosq;
    int rc;

    mosquitto_lib_init();

    mosq = mosquitto_new("client_c_1", true, NULL);
    if(!mosq) {
        fprintf(stderr, "[-] Error creating client\n");
        return 1;
    }

    // mosquitto_username_pw_set(mosq, "user", "password");

    mosquitto_connect_callback_set(mosq, on_connect);
    mosquitto_message_callback_set(mosq, on_message);
    mosquitto_disconnect_callback_set(mosq, on_disconnect);

    rc = mosquitto_connect(mosq, BROKER_HOST, BROKER_PORT, KEEPALIVE);
    if(rc != MOSQ_ERR_SUCCESS) {
        fprintf(stderr, "[-] Unable to connect (%s)\n", mosquitto_strerror(rc));
        return 1;
    }

    mosquitto_loop_start(mosq);

    while(1) {
        int choice;
        char topic[64];
        char message[256];

        printf("\n1. Publish\n");
        printf("2. Subscribe\n");
        printf("3. Exit\n");
        printf("> ");

        scanf("%d", &choice);
        getchar(); // limpiar buffer

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
            printf("[+] Subscribed to topic: %s\n", topic);
        }
        else if(choice == 3) {
            break;
        }
        else {
            printf("[-] Invalid option\n");
        }

        mosquitto_loop(mosq, 100, 1);
    }

    mosquitto_disconnect(mosq);
    mosquitto_loop_stop(mosq, true);
    mosquitto_destroy(mosq);
    mosquitto_lib_cleanup();

    return 0;
}