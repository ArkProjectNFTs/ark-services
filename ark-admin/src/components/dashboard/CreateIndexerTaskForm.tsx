"use client";

/* eslint-disable @typescript-eslint/no-misused-promises */
import { useEffect } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { Loader2 } from "lucide-react";
import { useForm } from "react-hook-form";
import { z } from "zod";

import { Button } from "~/components/ui/button";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "~/components/ui/form";
import { Input } from "~/components/ui/input";
import { api } from "~/trpc/react";
import { Checkbox } from "../ui/checkbox";
import { useToast } from "../ui/use-toast";
import { useNetwork } from "./NetworkProvider";
import { useTaskForm } from "./TaskFormProvider";

const createIndexerTaskFormSchema = z.object({
  from: z.string(),
  to: z.string(),
  numberOfTasks: z.string(),
  forceMode: z.boolean(),
  logLevel: z.string(),
});

type CreateIndexerTaskFormValues = z.infer<typeof createIndexerTaskFormSchema>;

const defaultValues: Partial<CreateIndexerTaskFormValues> = {
  from: "0",
  to: "1000",
  numberOfTasks: "3",
  forceMode: false,
  logLevel: "info",
};

export default function CreateIndexerTaskFrom() {
  const { network } = useNetwork();
  const { toast } = useToast();
  const { state } = useTaskForm();
  const { mutateAsync: spawnTasks, isLoading } =
    api.indexer.spawnTasks.useMutation({
      onSuccess: () => {
        toast({
          title: "Tasks created",
          description: "The tasks were created successfully",
        });
      },
      onError: (error) => {
        console.log(error);
        toast({
          title: "An error occured",
          description: "The tasks could not be created",
        });
      },
    });

  const form = useForm<CreateIndexerTaskFormValues>({
    resolver: zodResolver(createIndexerTaskFormSchema),
    defaultValues: { ...state, forceMode: false },
  });

  const from = form.watch("from");
  const to = form.watch("to");
  const count = parseInt(to) - parseInt(from) || 0;
  const isDisabled = count <= 0 || isLoading;

  async function onSubmit(data: CreateIndexerTaskFormValues) {
    await spawnTasks({
      from: parseInt(data.from),
      to: parseInt(data.to),
      numberOfTasks: parseInt(data.numberOfTasks),
      network,
      forceMode: data.forceMode,
    });
  }

  useEffect(() => {
    form.reset({ ...state });
  }, [state, form]);

  return (
    <div className="">
      <h3 className="text-2xl font-semibold tracking-tight">New Task</h3>
      <div className="mb-4 text-sm text-muted-foreground">
        Start indexing {count} blocks
      </div>
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <FormField
            control={form.control}
            name="from"
            render={({ field }) => (
              <FormItem>
                <FormLabel>From</FormLabel>
                <FormControl>
                  <Input placeholder="0" {...field} type="number" />
                </FormControl>
                <FormDescription>
                  The block number from which indexation begins.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="to"
            render={({ field }) => (
              <FormItem>
                <FormLabel>To</FormLabel>
                <FormControl>
                  <Input placeholder="100000" {...field} type="number" />
                </FormControl>
                <FormDescription>
                  The block number from which indexation ends.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="numberOfTasks"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Number of tasks</FormLabel>
                <FormControl>
                  <Input placeholder="3" {...field} type="number" />
                </FormControl>
                <FormDescription>Tasks to deploy for indexing</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="forceMode"
            render={({ field }) => (
              <FormItem>
                <div className="flex items-center space-x-2">
                  <Checkbox
                    id={field.name}
                    checked={field.value}
                    onCheckedChange={(value) =>
                      form.setValue(field.name, value as boolean)
                    }
                  />
                  <FormLabel htmlFor={field.name}>Force Mode</FormLabel>
                </div>
                <FormDescription>
                  Initiate re-indexing regardless of blocks previously indexed
                </FormDescription>
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="logLevel"
            render={({ field }) => (
              <FormItem>
                <FormLabel>Log Level</FormLabel>
                <FormControl>
                  <Input
                    {...field}
                    placeholder="Choose log level"
                    type="text"
                  />
                </FormControl>
                <FormMessage />
                <FormDescription>
                  Specify the desired logging level for the indexer task.
                </FormDescription>
              </FormItem>
            )}
          />
          <Button className="w-full" type="submit" disabled={isDisabled}>
            {isLoading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <>Create Tasks</>
            )}
          </Button>
        </form>
      </Form>
    </div>
  );
}
