/* eslint-disable @typescript-eslint/no-misused-promises */
import { zodResolver } from "@hookform/resolvers/zod";
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

interface CreateIndexerTaskFromProps {
  network: "mainnet" | "testnet";
}

const createIndexerTaskFormSchema = z.object({
  from: z.string(),
  to: z.string(),
  numberOfTasks: z.string(),
});

type CreateIndexerTaskFormValues = z.infer<typeof createIndexerTaskFormSchema>;

const defaultValues: Partial<CreateIndexerTaskFormValues> = {
  from: "0",
  to: "1000",
  numberOfTasks: "3",
};

export default function CreateIndexerTaskFrom(
  props: CreateIndexerTaskFromProps,
) {
  const { mutateAsync: spawnTasks } = api.indexer.spawnTasks.useMutation();
  const form = useForm<CreateIndexerTaskFormValues>({
    resolver: zodResolver(createIndexerTaskFormSchema),
    defaultValues,
  });

  form.handleSubmit;

  async function onSubmit(data: CreateIndexerTaskFormValues) {
    await spawnTasks({
      from: parseInt(data.from),
      to: parseInt(data.to),
      numberOfTasks: parseInt(data.numberOfTasks),
      network: props.network,
    });
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
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

        <Button type="submit">Create Tasks</Button>
      </form>
    </Form>
  );
}
